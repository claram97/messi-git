use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Error, Read, Write},
    net::TcpStream,
    path::PathBuf,
    str::from_utf8,
    vec,
};

use crate::{hash_object, tree_handler::Tree, packfile_handler::{PackfileEntry, ObjectType}};
use crate::packfile_handler::Packfile;
use crate::cat_file;

// multi_ack_detailed side-band-64k thin-pack include-tag ofs-delta deepen-since deepen-not

const VERSION: &str = "1";
const GIT_UPLOAD_PACK: &str = "git-upload-pack";
const GIT_RECEIVE_PACK: &str = "git-receive-pack";
const CAPABILITIES_UPLOAD: &str = "multi_ack side-band-64k ofs-delta";
#[derive(Debug)]
pub struct Client {
    git_dir: String,
    address: String,
    repository: String,
    host: String,
    socket: Option<TcpStream>,
    refs: HashMap<String, String>,
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
    pub fn new(address: &str, repository: &str, host: &str) -> io::Result<Self> {
        let refs = HashMap::new();
        let repository = repository.to_owned();
        let host = host.to_owned();
        let address = address.to_owned();
        Ok(Self {
            git_dir: String::new(),
            address,
            repository,
            host,
            socket: None,
            refs,
        })
    }

    // Establish a connection with the server and asks for the refs in the remote.
    // A hashmap with the path of the refs as keys and the last commit hash as values is returned.
    //
    // Leaves the connection opened
    // May fail due to I/O errors
    pub fn get_refs(&mut self) -> io::Result<Vec<String>> {
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.wait_refs()?;
        self.flush()?;
        Ok(self.refs.keys().map(String::from).collect())
    }

    // REVISAR: deberia ser como el upload-pack the git
    pub fn upload_pack(&mut self, wanted_ref: Option<&str>, git_dir: &str) -> io::Result<()> {
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.git_dir = git_dir.to_string();
        self.wait_refs()?;

        if let Some(wanted_ref) = wanted_ref {
            if let Some(hash) = self.want_ref(wanted_ref)? {
                self.update_fetch_head(&hash)?;
                self.update_remote(wanted_ref, &hash)?;
                self.wait_packfile()?;
            }
        }
        Ok(())
    }

    pub fn receive_pack(&mut self, pushing_ref: &str, git_dir: &str) -> io::Result<()> {
        self.connect()?;
        self.initiate_connection(GIT_RECEIVE_PACK)?;
        self.flush()?;
        self.git_dir = git_dir.to_string();

        // ya se que tiene el servidor
        self.wait_refs()?;

        let local_refs = get_refs_heads(&self.git_dir)?;
        let new_hash = match local_refs.get(pushing_ref) {
            Some(hash) => hash,
            None => return Err(Error::new(io::ErrorKind::NotFound, "Ref not found")),
        };

        let prev_hash = match self.refs.get(pushing_ref) {
            Some(hash) => hash.clone(),
            None => String::new(),
        };

        if &prev_hash == new_hash {
            println!("Already up to date.");
            return Ok(());
        }

        if prev_hash.is_empty() {
            self.receive_pack_create(pushing_ref, new_hash)?;
        } else {
            self.receive_pack_update(pushing_ref, &prev_hash, new_hash)?;
        };

        // self.update_remote(pushing_ref, &new_hash)?;
        Ok(())
    }

    fn receive_pack_create(&mut self, pushing_ref: &str, hash: &str) -> io::Result<()> {
        self.receive_pack_update(pushing_ref, "0", hash)
    }

    fn receive_pack_update(
        &mut self,
        _pushing_ref: &str,
        prev_hash: &str,
        new_hash: &str,
    ) -> io::Result<()> {
        get_missing_objects_from(new_hash, prev_hash, &self.git_dir)?;
        Ok(())
    }

    // Establish the first conversation with the server
    // Lets the server know that an upload-pack is requested
    fn initiate_connection(&mut self, command: &str) -> io::Result<()> {
        let mut command = format!("{} /{}", command, self.repository);

        command = format!("{}\0host={}\0", command, self.host);

        command = format!("{}\0version={}\0", command, VERSION);

        let pkt_command = pkt_line(&command);

        println!("Enviando al socket: {:?}", &pkt_command);

        self.send(&pkt_command)?;
        println!("Termino de enviar al socket");
        Ok(())
    }

    // Auxiliar function. Waits for the refs and loads them in self
    // Should be called only aftes: initiate_connection
    fn wait_refs(&mut self) -> io::Result<()> {
        self.refs.clear();
        let (_, _) = read_pkt_line(self.socket()?)?;
        let (mut size, mut line) = read_pkt_line(self.socket()?)?;

        while size > 0 {
            if let Some((hash, mut ref_path)) = line.split_once(' ') {
                if let Some((head, _capabilities)) = ref_path.split_once('\0') {
                    ref_path = head;
                }
                self.refs
                    .insert(ref_path.trim().to_string(), hash.to_string());
            }
            (size, line) = read_pkt_line(self.socket()?)?;
        }
        Ok(())
    }

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
        write!(self.socket()?, "{}", message)
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

    // Updates remote head with the fetched hash
    // REVISAR: ver que onda de donde saco el remote
    fn update_remote(&self, remote_ref: &str, hash: &str) -> io::Result<()> {
        let pathbuf = PathBuf::from(&self.git_dir);
        let remote = pathbuf.join("refs").join("remotes").join("origin");
        fs::create_dir_all(&remote)?;
        let remote = remote.join(remote_ref);
        let mut file = fs::File::create(remote)?;
        writeln!(file, "{}", hash)
    }

    // Tells the server which refs are required
    // If the provided ref name does not exist in remote, then an error is returned.
    fn want_ref(&mut self, wanted_ref: &str) -> io::Result<Option<String>> {
        println!("Pido: {}", wanted_ref);

        let hash = match self.refs.get(wanted_ref) {
            Some(hash) => hash.clone(),
            None => {
                return Err(Error::new(
                    io::ErrorKind::NotFound,
                    "Ref not found in remote",
                ))
            }
        };

        let local_refs = get_refs_heads(&self.git_dir)?;

        if let Some(local_hash) = local_refs.get(wanted_ref) {
            if &hash == local_hash {
                println!("Already up to date.");
                return Ok(None);
            }
        }

        let want = format!("want {} {}\n", hash, CAPABILITIES_UPLOAD);
        let want = pkt_line(&want);
        dbg!(&want);
        self.send(&want)?;

        self.flush()?;

        self.send_haves(local_refs)?;
        // std::thread::sleep(std::time::Duration::from_secs(5));
        // self.wait_packfile()?;
        self.done()?;
        Ok(Some(hash))
    }

    fn send_haves(&mut self, local_refs: HashMap<String, String>) -> io::Result<()> {
        if !local_refs.is_empty() {
            for hash in local_refs.values() {
                let have = format!("have {}\n", hash);
                let have = pkt_line(&have);
                // dbg!(&have);
                self.send(&have)?;
            }
            self.flush()?;
        }
        Ok(())
    }

    // Auxiliar function. Reads the socket until a 'flush' signal is read
    fn wait_packfile(&mut self) -> io::Result<()> {
        let socket = self.socket()?;
        while let Ok((size, bytes)) = read_pkt_line_bytes(socket) {
            if size < 4 {
                break;
            }
            if bytes[0] == 1 {
                let packfile = Packfile::new(&bytes[..])?;
                for entry in packfile {
                    let entry = entry?;
                    hash_object::store_string_to_file(&entry.content, &self.git_dir, &entry.obj_type.to_string())?;
                }
                return Ok(());
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

fn connection_not_established_error() -> Error {
    Error::new(
        io::ErrorKind::BrokenPipe,
        "The operation failed because the connection was not established.",
    )
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

// Auxiliar function which get refs under refs/heads
fn get_refs_heads(git_dir: &str) -> io::Result<HashMap<String, String>> {
    let mut refs = HashMap::new();
    let pathbuf = PathBuf::from(git_dir);
    let heads = pathbuf.join("refs").join("heads");
    for entry in fs::read_dir(&heads)? {
        let filename = entry?.file_name().to_string_lossy().to_string();
        let path = heads.join(filename);
        let hash: String = fs::read_to_string(&path)?.trim().into();
        let ref_path = path
            .to_string_lossy()
            .to_string()
            .split_once('/')
            .ok_or(Error::new(io::ErrorKind::Other, "Unknown error"))?
            .1
            .to_string();

        refs.insert(ref_path, hash);
    }
    let head = pathbuf.join("HEAD");
    if head.exists() {
        let head_content: String = fs::read_to_string(head)?.trim().into();
        match head_content.split_once(": ") {
            Some((_, branch)) => {
                if let Some(hash) = refs.get(branch) {
                    refs.insert("HEAD".to_string(), hash.trim().into());
                }
            }
            None => {
                refs.insert("HEAD".to_string(), head_content);
            }
        };
    }
    Ok(refs)
}

fn get_missing_objects_from(
    new_hash: &str,
    prev_hash: &str,
    git_dir: &str,
) -> io::Result<Vec<(ObjectType, String)>> {
    let mut missing = vec![];
    let hash = new_hash;
    while let Ok(commit) = Commit::new(hash, git_dir) {
        if hash == prev_hash {
            break
        }
        missing.push((ObjectType::Commit, commit.hash.to_string()));
        missing.push((ObjectType::Tree, commit.tree.to_string()));
        let tree_objects = get_objects_from_tree(&commit.tree, git_dir)?;
        tree_objects.iter().for_each(|obj| missing.push((obj.0, obj.1.clone())));

        for parent in commit.parent {
            let _missing = get_missing_objects_from(&parent, prev_hash, git_dir)?;
            _missing.iter().for_each(|p| missing.push((p.0, p.1.clone())));
        }
    }

    Ok(missing)
}

#[derive(Debug, Default)]
struct Commit {
    hash: String,
    tree: String,
    parent: Vec<String>
}

impl Commit {
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
            None => Err(Error::new(io::ErrorKind::InvalidData, format!("Commit: {}", hash))),
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

fn get_objects_from_tree(hash: &str, git_dir: &str) -> io::Result<Vec<(ObjectType, String)>> {

    let mut objects = vec![];
    let content = cat_file::cat_file_return_content(hash, git_dir)?;
    
    for line in content.lines() {
        let split = line.split(' ').skip(1).take(2);
        let val: Vec<&str>= split.collect();
        let t = val.get(0).ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid data in tree"))?;
        let hash = val.get(1).ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid data in tree"))?;
        let t = ObjectType::try_from(*t)?;
        objects.push((t, hash.to_string()))
    }

    Ok(objects)
}
