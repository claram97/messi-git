use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Error, Write},
    net::TcpStream,
    path::PathBuf,
};

use crate::{packfile_handler, server_utils::*};

const VERSION: &str = "1";
const GIT_UPLOAD_PACK: &str = "git-upload-pack";
const GIT_RECEIVE_PACK: &str = "git-receive-pack";
const CAPABILITIES_UPLOAD: &str = "multi_ack side-band-64k ofs-delta";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000";

#[derive(Debug, Default)]
pub struct Client {
    address: String,
    repository: String,
    host: String,
    socket: Option<TcpStream>,
    git_dir: String,
    remote: String,
    server_refs: HashMap<String, String>,
}

/// This is a git client that is able to connect to a git server
/// using the git protocol.
impl Client {
    /// Creates client which will connect with a server (assuming its a git server)
    ///
    /// Parameters:
    ///     - address: address to establish a tcp connection
    ///     - repository: name of the repository in the remote
    ///     - host: name of the host. e.g. localhost
    pub fn new(address: &str, repository: &str, host: &str) -> Self {
        let mut client = Self::default();
        client.repository = repository.to_owned();
        client.host = host.to_owned();
        client.address = address.to_owned();
        client
    }

    // Establish a connection with the server and asks for the refs in the remote.
    // A hashmap with the path of the refs as keys and the last commit hash as values is returned.
    //
    // Leaves the connection opened
    // May fail due to I/O errors
    pub fn get_server_refs(&mut self) -> io::Result<HashMap<String, String>> {
        self.clear();
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.wait_server_refs()?;
        self.flush()?;
        Ok(self.server_refs.clone())
    }

    /// Establish a connection with the server and asks for the refs in the remote.
    /// If the local remote refs are up to date, then nothing is done.
    /// Else, the server is asked for the missing objects and a packfile is unpacked.
    /// Then the remote refs are updated.
    ///
    /// Parameters:
    ///    - wanted_branchs: vector with the names of the branchs to fetch
    ///    - git_dir: path to the git directory
    ///    - remote: name of the remote
    pub fn upload_pack(
        &mut self,
        wanted_branchs: Vec<&str>, // recibir vector con varias branchs
        git_dir: &str,
        remote: &str,
    ) -> io::Result<()> {
        self.clear();
        self.git_dir = git_dir.to_string();
        self.remote = remote.to_string();
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.wait_server_refs()?;

        let fetched_remotes_refs = self.want_branchs(wanted_branchs)?;
        if fetched_remotes_refs.is_empty() {
            println!("Already up to date.");
            return Ok(());
        }
        self.wait_and_unpack_packfile()?;
        for (branch, hash) in fetched_remotes_refs {
            self.update_remote(&branch, &hash)?;
        }
        Ok(())
    }

    /// Establish a connection with the server and sends the local refs to the server.
    /// If the remote refs are up to date, then nothing is done.
    ///
    /// Refs can be updated, created or deleted. However, deletion is not implemented yet.
    ///
    /// Parameters:
    ///   - branch: name of the branch to push
    ///   - git_dir: path to the git directory
    pub fn receive_pack(&mut self, branch: &str, git_dir: &str) -> io::Result<()> {
        self.clear();
        self.connect()?;
        self.initiate_connection(GIT_RECEIVE_PACK)?;
        self.git_dir = git_dir.to_string();
        let pushing_ref = get_head_from_branch(git_dir, branch)?;
        self.wait_server_refs()?;
        let client_heads_refs = get_head_refs(&self.git_dir)?;
        let new_hash = match client_heads_refs.get(branch) {
            Some(hash) => hash,
            None => {
                return Err(Error::new(
                    io::ErrorKind::NotFound,
                    format!("Ref not found in local: {}", pushing_ref),
                ))
            }
        };
        let prev_hash = match self.server_refs.get(&pushing_ref) {
            Some(hash) => hash.clone(),
            None => String::new(),
        };
        if &prev_hash == new_hash {
            println!("Already up to date.");
            return Ok(());
        }
        if prev_hash.is_empty() {
            self.receive_pack_create(&pushing_ref, new_hash)
        } else {
            self.receive_pack_update(&pushing_ref, &prev_hash, new_hash)
        }
    }

    // Clears the client state
    fn clear(&mut self) {
        self.git_dir = String::new();
        self.remote = String::new();
        self.server_refs.clear();
    }

    // Connects to the server and returns a Tcp socket
    fn connect(&mut self) -> io::Result<()> {
        self.socket = Some(TcpStream::connect(&self.address)?);
        Ok(())
    }

    // Establish the first conversation with the server
    // Lets the server know that an upload-pack is requested
    fn initiate_connection(&mut self, command: &str) -> io::Result<()> {
        let mut command = format!("{} /{}", command, self.repository);

        command = format!("{}\0host={}\0", command, self.host);

        command = format!("{}\0version={}\0", command, VERSION);

        let pkt_command = pkt_line(&command);

        self.send(&pkt_command)
    }

    // Auxiliar function. Waits for the refs and loads them in self
    // Should be called only aftes: initiate_connection
    fn wait_server_refs(&mut self) -> io::Result<()> {
        let (_, _) = read_pkt_line(self.socket()?)?;
        let (mut size, mut line) = read_pkt_line(self.socket()?)?;

        while size > 0 {
            if let Some((hash, mut ref_path)) = line.split_once(' ') {
                if let Some((head, _capabilities)) = ref_path.split_once('\0') {
                    ref_path = head;
                }
                self.server_refs
                    .insert(ref_path.trim().to_string(), hash.to_string());
            }
            (size, line) = read_pkt_line(self.socket()?)?;
        }
        Ok(())
    }

    // Auxiliar function. Given a vector of branchs, will ask the server for the missing objects
    //
    // Will fail if the server does not have the wanted branchs
    //
    // If the server has the wanted branchs, then it will send the 'want' and 'have' messages
    fn want_branchs(&mut self, branchs: Vec<&str>) -> io::Result<HashMap<String, String>> {
        let mut fetched_remotes_refs = HashMap::new();

        let client_remotes_refs = get_remote_refs(&self.git_dir, &self.remote)?;
        for branch in branchs {
            let wanted_ref = get_head_from_branch(&self.git_dir, branch)?;
            let hash = match self.server_refs.get(&wanted_ref) {
                Some(hash) => hash.clone(),
                None => {
                    return Err(Error::new(
                        io::ErrorKind::NotFound,
                        format!("Ref not found in remote: {}", wanted_ref),
                    ))
                }
            };
            if let Some(local_hash) = client_remotes_refs.get(branch) {
                if &hash == local_hash {
                    continue;
                }
            }
            fetched_remotes_refs.insert(branch.to_string(), hash);
        }
        if !fetched_remotes_refs.is_empty() {
            self.send_wants(fetched_remotes_refs.values().collect())?;
            self.send_haves(client_remotes_refs)?;
            self.done()?;
        }
        Ok(fetched_remotes_refs)
    }

    // Sends a 'want' message to the server for each hash in the vector
    fn send_wants(&mut self, hashes: Vec<&String>) -> io::Result<()> {
        let first_want = format!("want {} {}\n", hashes[0], CAPABILITIES_UPLOAD);
        let first_want = pkt_line(&first_want);
        self.send(&first_want)?;

        for hash in hashes.get(1..).unwrap_or(&[]) {
            let want = format!("want {}\n", hash);
            let want = pkt_line(&want);
            self.send(&want)?;
        }
        self.flush()
    }

    // Sends a 'have' message to the server for each hash in the hashmap
    fn send_haves(&mut self, local_refs: HashMap<String, String>) -> io::Result<()> {
        if local_refs.is_empty() {
            return Ok(());
        }
        for hash in local_refs.values() {
            let have = format!("have {}\n", hash);
            let have = pkt_line(&have);
            self.send(&have)?;
        }
        self.flush()
    }

    // Waits for the server to send a packfile
    // After receiving it, it is unpacked and stored in the git_dir
    fn wait_and_unpack_packfile(&mut self) -> io::Result<()> {
        let socket = self.socket()?;
        while let Ok((size, bytes)) = read_pkt_line_bytes(socket) {
            if size < 4 {
                break;
            }
            if bytes[0] == 1 {
                return packfile_handler::unpack_packfile(&bytes[..], &self.git_dir);
            }
        }
        Err(Error::new(io::ErrorKind::NotFound, "Packfile not found"))
    }

    // Updates remote ref with the fetched hash
    // If the ref does not exist, then it is created
    fn update_remote(&self, remote_ref: &str, hash: &str) -> io::Result<()> {
        let pathbuf = PathBuf::from(&self.git_dir);
        let remote = pathbuf.join("refs").join("remotes").join(&self.remote);
        fs::create_dir_all(&remote)?;
        let remote = remote.join(remote_ref);
        let mut file = fs::File::create(remote)?;
        writeln!(file, "{}", hash)
    }

    // Sends a 'create' message to the server using 'update' message
    fn receive_pack_create(&mut self, pushing_ref: &str, hash: &str) -> io::Result<()> {
        self.receive_pack_update(pushing_ref, ZERO_HASH, hash)
    }

    // Sends a 'update' message to the server
    // Then, it sends the missing objects to the server in a packfile
    fn receive_pack_update(
        &mut self,
        pushing_ref: &str,
        prev_hash: &str,
        new_hash: &str,
    ) -> io::Result<()> {
        let update = format!("{} {} {}\0", prev_hash, new_hash, pushing_ref);
        self.send(&pkt_line(&update))?;
        self.flush()?;

        let mut haves = HashSet::new();
        haves.insert(prev_hash.to_string());

        let missing_objects = get_missing_objects_from(new_hash, &haves, &self.git_dir)?;
        let packfile = packfile_handler::create_packfile_from_set(missing_objects, &self.git_dir)?;
        let packfile: Vec<u8> = [vec![1], packfile].concat();
        self.send_bytes(&pkt_line_bytes(&packfile))?;
        Ok(())
    }

    // Returns a mutable reference to the socket if it has a established connection
    fn socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.socket {
            Some(ref mut socket) => Ok(socket),
            None => Err(connection_not_established_error()),
        }
    }

    fn end_connection(&mut self) -> io::Result<()> {
        self.flush()?;
        self.socket = None;
        Ok(())
    }

    // Sends a message through the socket
    fn send(&mut self, message: &str) -> io::Result<()> {
        dbg!(message);
        write!(self.socket()?, "{}", message)
    }

    // Sends a message through the socket as bytes
    fn send_bytes(&mut self, content: &[u8]) -> io::Result<()> {
        dbg!("Sending bytes...");
        self.socket()?.write_all(content)
    }

    // Sends a 'flush' signal to the server
    fn flush(&mut self) -> io::Result<()> {
        self.send("0000")
    }

    // Sends a 'done' signal to the server
    fn done(&mut self) -> io::Result<()> {
        self.send("0009done\n")
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.end_connection();
    }
}
