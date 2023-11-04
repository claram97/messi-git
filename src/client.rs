use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Error, Read, Write},
    net::TcpStream,
    path::PathBuf,
    str::from_utf8,
    vec,
};

use crate::cat_file;
use crate::packfile_handler::Packfile;
use crate::{
    hash_object,
    packfile_handler::{self, ObjectType},
};

// multi_ack_detailed side-band-64k thin-pack include-tag ofs-delta deepen-since deepen-not

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
    ///     - host: REVISAR (no se si es si o si el mismo que address)
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
    pub fn get_server_refs(&mut self) -> io::Result<Vec<String>> {
        self.clear();
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.wait_server_refs()?;
        self.flush()?;
        Ok(self.server_refs.keys().map(String::from).collect())
    }

    /// Establish a connection with the server and asks for the refs in the remote.
    /// If the local remote refs are up to date, then nothing is done.
    /// Else, the server is asked for the missing objects and a packfile unpacked.
    /// Then the remote refs are updated.
    pub fn upload_pack(
        &mut self,
        wanted_branch: &str,
        git_dir: &str,
        remote: &str,
    ) -> io::Result<()> {
        self.clear();
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.git_dir = git_dir.to_string();
        self.remote = remote.to_string();
        self.wait_server_refs()?;

        if let Some(hash) = self.want_branch(&wanted_branch)? {
            self.update_fetch_head(&hash)?;
            self.wait_and_unpack_packfile()?;
            self.update_remote(&wanted_branch, &hash)?;
        }

        Ok(())
    }

    pub fn receive_pack(&mut self, branch: &str, git_dir: &str) -> io::Result<()> {
        self.clear();
        self.connect()?;
        self.initiate_connection(GIT_RECEIVE_PACK)?;
        self.git_dir = git_dir.to_string();

        // let pushing_ref = if branch == "HEAD" {
        //     format!("refs/heads/{}", get_head_from_branch(&self.git_dir, branch)?)
        // } else {
        //     format!("refs/heads/{}", branch)
        // };

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

    fn receive_pack_create(&mut self, pushing_ref: &str, hash: &str) -> io::Result<()> {
        self.receive_pack_update(pushing_ref, ZERO_HASH, hash)
    }

    fn receive_pack_update(
        &mut self,
        pushing_ref: &str,
        prev_hash: &str,
        new_hash: &str,
    ) -> io::Result<()> {
        let update = format!("{} {} {}\0", prev_hash, new_hash, pushing_ref);
        self.send(&pkt_line(&update))?;
        self.flush()?;

        let missing_objects = get_missing_objects_from(new_hash, prev_hash, &self.git_dir)?;
        let packfile = packfile_handler::create_packfile_from_set(missing_objects, &self.git_dir)?;
        self.send_bytes(packfile.as_slice())?;
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

    // Clears the client state
    fn clear(&mut self) {
        self.git_dir = String::new();
        self.remote = String::new();
        self.server_refs.clear();
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

    // Returns a mutable reference to the socket if it has a established connection
    fn socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.socket {
            Some(ref mut socket) => Ok(socket),
            None => Err(connection_not_established_error()),
        }
    }

    // Connects to the server and returns a Tcp socket
    fn connect(&mut self) -> io::Result<()> {
        self.socket = Some(TcpStream::connect(&self.address)?);
        Ok(())
    }

    fn end_connection(&mut self) -> io::Result<()> {
        self.flush()?;
        self.socket = None;
        Ok(())
    }

    // Sends a message throw the socket
    fn send(&mut self, message: &str) -> io::Result<()> {
        dbg!(message);
        write!(self.socket()?, "{}", message)
    }

    // Sends a message throw the socket as bytes
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

    // Updates FETCH_HEAD file overwritting it
    fn update_fetch_head(&self, hash: &str) -> io::Result<()> {
        let pathbuf = PathBuf::from(&self.git_dir);
        let fetch_head = pathbuf.join("FETCH_HEAD");
        let mut fetch_head = fs::File::create(fetch_head)?;
        writeln!(fetch_head, "{}", hash)
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

    // Tells the server which refs are required
    // If the provided ref name does not exist in remote, then an error is returned.
    fn want_branch(&mut self, branch: &str) -> io::Result<Option<String>> {
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

        let client_remotes_refs = get_remote_refs(&self.git_dir, &self.remote)?;
        if let Some(local_hash) = client_remotes_refs.get(branch) {
            if &hash == local_hash {
                println!("Already up to date.");
                return Ok(None);
            }
        }
        let want = format!("want {} {}\n", hash, CAPABILITIES_UPLOAD);
        let want = pkt_line(&want);
        self.send(&want)?;

        self.flush()?;
        self.send_haves(client_remotes_refs)?;
        self.done()?;
        Ok(Some(hash))
    }

    fn send_haves(&mut self, local_refs: HashMap<String, String>) -> io::Result<()> {
        if !local_refs.is_empty() {
            for hash in local_refs.values() {
                let have = format!("have {}\n", hash);
                let have = pkt_line(&have);
                self.send(&have)?;
            }
            self.flush()?;
        }
        Ok(())
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
                return unpack_packfile(&bytes[..], &self.git_dir)
            }
        }
        Err(Error::new(io::ErrorKind::NotFound, "Packfile not found"))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.end_connection();
    }
}

// De aca abajo son funciones que sirven para el server tambien

fn connection_not_established_error() -> Error {
    Error::new(
        io::ErrorKind::BrokenPipe,
        "The operation failed because the connection was not established.",
    )
}

fn unpack_packfile(packfile: &[u8], git_dir: &str) -> io::Result<()> {
    let packfile = Packfile::reader(packfile)?;
    for entry in packfile {
        let entry = entry?;
        hash_object::store_bytes_array_to_file(
            entry.content,
            &git_dir,
            &entry.obj_type.to_string(),
        )?;
    }
    Ok(())
}

// Read a line in PKT format in a TcpStream
// Returns the size of the line and its content
fn read_pkt_line(socket: &mut TcpStream) -> io::Result<(usize, String)> {
    let (size, bytes) = read_pkt_line_bytes(socket)?;
    let line = from_utf8(&bytes).unwrap_or_default().to_string();
    Ok((size, line))
}

fn read_pkt_line_bytes(socket: &mut TcpStream) -> io::Result<(usize, Vec<u8>)> {
    let mut buf = vec![0u8; 4];
    socket.read_exact(&mut buf)?;

    let size = from_utf8(&buf).unwrap_or_default();
    let size = usize::from_str_radix(size, 16).unwrap_or(0);

    if size < 4 {
        return Ok((size, vec![]));
    }

    let mut buf = vec![0u8; size - 4];
    socket.read_exact(&mut buf)?;
    Ok((size, buf))
}

// Given a text to send a git client/server, this function transform it to a
// string in PKT format
fn pkt_line(line: &str) -> String {
    let len = line.len() + 4; // len
    let mut len_hex = format!("{len:x}");
    while len_hex.len() < 4 {
        len_hex = "0".to_owned() + &len_hex
    }
    len_hex + line
}

fn get_head_from_branch(git_dir: &str, branch: &str) -> io::Result<String> {
    if branch != "HEAD" {
        return Ok(format!("refs/heads/{}", branch));
    }

    let head = PathBuf::from(git_dir).join("HEAD");
    let content = fs::read_to_string(head)?;
    let (_, head) = content.rsplit_once(": ").ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Invalid data HEAD. Must have ref for fetch: {}", content),
    ))?;
    Ok(head.trim().to_string())
}
// Auxiliar function which get refs under refs/heads
fn get_head_refs(git_dir: &str) -> io::Result<HashMap<String, String>> {
    let pathbuf = PathBuf::from(git_dir);
    let heads = pathbuf.join("refs").join("heads");
    get_refs(heads)
}

// Auxiliar function which get refs under refs/heads
fn get_remote_refs(git_dir: &str, remote: &str) -> io::Result<HashMap<String, String>> {
    let pathbuf = PathBuf::from(git_dir);
    let remotes = pathbuf.join("refs").join("remotes").join(remote);
    get_refs(remotes)
}

fn get_refs(refs_path: PathBuf) -> io::Result<HashMap<String, String>> {
    let mut refs = HashMap::new();
    for entry in fs::read_dir(&refs_path)? {
        let filename = entry?.file_name().to_string_lossy().to_string();
        let path = refs_path.join(&filename);
        let hash: String = fs::read_to_string(&path)?.trim().into();
        refs.insert(filename, hash);
    }
    Ok(refs)
}

fn get_missing_objects_from(
    new_hash: &str,
    prev_hash: &str,
    git_dir: &str,
) -> io::Result<HashSet<(ObjectType, String)>> {
    let mut missing: HashSet<(ObjectType, String)> = HashSet::new();

    if new_hash == prev_hash {
        return Ok(missing);
    }

    if let Ok(commit) = CommitHashes::new(new_hash, git_dir) {
        missing.insert((ObjectType::Commit, commit.hash.to_string()));

        let tree_objects = get_objects_tree_objects(&commit.tree, git_dir)?;
        missing.extend(tree_objects);

        for parent in commit.parent {
            let _missing = get_missing_objects_from(&parent, prev_hash, git_dir)?;
            missing.extend(_missing);
        }
    }

    Ok(missing)
}

#[derive(Debug, Default)]
struct CommitHashes {
    hash: String,
    tree: String,
    parent: Vec<String>,
}

impl CommitHashes {
    fn new(hash: &str, git_dir: &str) -> io::Result<Self> {
        let commit_content = cat_file::cat_file_return_content(hash, git_dir)?;
        let header_lines = commit_content.lines().position(|line| line.is_empty());
        match header_lines {
            Some(n) => {
                let mut commit = Self::default();
                for line in commit_content.lines().take(n) {
                    commit.parse_commit(line)
                }
                commit.hash = hash.to_string();
                Ok(commit)
            }
            None => Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Commit: {}", hash),
            )),
        }
    }

    fn parse_commit(&mut self, line: &str) {
        match line.split_once(' ') {
            Some(("tree", hash)) => self.tree = hash.to_string(),
            Some(("parent", hash)) => self.parent.push(hash.to_string()),
            _ => {}
        }
    }
}

fn get_objects_tree_objects(
    hash: &str,
    git_dir: &str,
) -> io::Result<HashSet<(ObjectType, String)>> {
    let mut objects: HashSet<(ObjectType, String)> = HashSet::new();
    objects.insert((ObjectType::Tree, hash.to_string()));
    let content = cat_file::cat_tree(hash, git_dir)?;

    for (mode, _, hash) in content {
        if mode == "040000" {
            let tree_objects = get_objects_tree_objects(&hash, git_dir)?;
            objects.extend(tree_objects);
        } else {
            objects.insert((ObjectType::Blob, hash.to_string()));
        };
    }

    Ok(objects)
}
