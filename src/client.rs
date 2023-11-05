use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, BufReader, Error, Read, Write},
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
#[derive(Debug, Default)]
pub struct Client {
    git_dir: String,
    address: String,
    repository: String,
    host: String,
    remote: String,
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
    pub fn get_refs(&mut self) -> io::Result<Vec<String>> {
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.wait_refs()?;
        self.flush()?;
        Ok(self.refs.keys().map(String::from).collect())
    }

    // REVISAR: deberia ser como el upload-pack the git
    pub fn upload_pack(
        &mut self,
        wanted_branch: &str,
        git_dir: &str,
        remote: &str,
    ) -> io::Result<String> {
        self.connect()?;
        self.initiate_connection(GIT_UPLOAD_PACK)?;
        self.git_dir = git_dir.to_string();
        self.remote = remote.to_string();
        self.wait_refs()?;

        let mut branch = wanted_branch.to_string();

        let wanted_ref = if wanted_branch == "HEAD" {
            branch = get_head_branch(git_dir)?;
            format!("refs/heads/{}", branch)
        } else {
            format!("refs/heads/{}", wanted_branch)
        };

        if let Some(hash) = self.want_ref(&wanted_ref)? {
            self.update_fetch_head(&hash)?;
            self.update_remote(&branch, &hash)?;
            self.wait_packfile()?;
            return Ok(hash);
        }

        Ok(String::new())
    }

    /// Initiates the "receive-pack" process for pushing updates to the remote Git repository.
    ///
    /// This function establishes a connection to the remote Git repository, negotiates the "receive-pack" process,
    /// and sends updates for the specified reference. It checks if the reference exists locally, calculates the
    /// difference in commits, and sends the updates to the remote repository. If the local reference is already up
    /// to date, no updates are sent.
    ///
    /// # Arguments
    ///
    /// * `self`: A mutable reference to a `Client` instance, used to send updates to the remote repository.
    /// * `pushing_ref`: The reference (branch) to be pushed to the remote repository.
    /// * `git_dir`: The path to the local directory containing the Git repository.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
    pub fn receive_pack(&mut self, pushing_ref: &str, git_dir: &str) -> io::Result<()> {
        self.connect()?;
        self.initiate_connection(GIT_RECEIVE_PACK)?;
        // self.flush()?;
        self.git_dir = git_dir.to_string();

        // ya se que tiene el servidor
        self.wait_refs()?;
        let local_refs = get_refs_heads(&self.git_dir)?;
        let new_hash = match local_refs.get(pushing_ref) {
            Some(hash) => hash,
            None => {
                return Err(Error::new(
                    io::ErrorKind::NotFound,
                    format!("Ref not found in local: {}", pushing_ref),
                ))
            }
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

        let mut reader = BufReader::new(self.socket()?);
        let mut res = String::new();
        reader.read_to_string(&mut res)?;
        dbg!(res);
        // self.update_remote(pushing_ref, &new_hash)?;
        Ok(())
    }

    /// Initiates the "receive-pack" process for creating or updating a reference in the remote Git repository.
    ///
    /// This function initiates the "receive-pack" process for creating or updating a reference (branch) in the
    /// remote Git repository. It prepares and sends the necessary data, including the previous and new commit
    /// hashes and the reference to be updated. For creating a reference, pass "0" as the previous hash.
    ///
    /// # Arguments
    ///
    /// * `self`: A mutable reference to a `Client` instance, used to send updates to the remote repository.
    /// * `pushing_ref`: The reference (branch) to be created or updated in the remote repository.
    /// * `hash`: The new commit hash associated with the reference.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
    fn receive_pack_create(&mut self, pushing_ref: &str, hash: &str) -> io::Result<()> {
        self.receive_pack_update(pushing_ref, "0", hash)
    }

    /// Initiates the "receive-pack" process for updating a reference in the remote Git repository.
    ///
    /// This function initiates the "receive-pack" process for updating a reference (branch) in the remote Git repository.
    /// It prepares and sends the necessary data, including the previous and new commit hashes and the reference to be updated.
    ///
    /// # Arguments
    ///
    /// * `self`: A mutable reference to a `Client` instance, used to send updates to the remote repository.
    /// * `pushing_ref`: The reference (branch) to be updated in the remote repository.
    /// * `prev_hash`: The previous commit hash associated with the reference.
    /// * `new_hash`: The new commit hash to update the reference.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
    fn receive_pack_update(
        &mut self,
        pushing_ref: &str,
        prev_hash: &str,
        new_hash: &str,
    ) -> io::Result<()> {
        let update = format!("{} {} {}\0", prev_hash, new_hash, pushing_ref);
        // dbg!("Sleeping...");
        // std::thread::sleep(std::time::Duration::from_secs(5));
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

    /// Returns a mutable reference to the underlying `TcpStream` used for communication.
    ///
    /// This method retrieves a mutable reference to the underlying `TcpStream` associated with the current
    /// `Client` instance, which is used for communication with the remote Git repository. The method ensures
    /// that a connection to the remote server has been established before returning the reference to the stream.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the mutable reference to the `TcpStream` in case of a successful connection,
    /// or an error in case the connection has not been established. The result is wrapped in an `io::Result<&mut TcpStream>`.
    ///
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

    /// Ends the connection to the remote Git repository and closes the underlying socket.
    ///
    /// This method is responsible for finalizing the connection to the remote Git repository by ensuring that
    /// any pending data is flushed and the underlying socket is closed. It effectively ends the communication
    /// session with the remote repository.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
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

    // Updates remote head with the fetched hash
    // REVISAR: ver que onda de donde saco el remote
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
    fn want_ref(&mut self, wanted_ref: &str) -> io::Result<Option<String>> {
        println!("Pido: {}", wanted_ref);

        let hash = match self.refs.get(wanted_ref) {
            Some(hash) => hash.clone(),
            None => {
                return Err(Error::new(
                    io::ErrorKind::NotFound,
                    format!("Ref not found in remote: {}", wanted_ref),
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
        self.send(&want)?;

        self.flush()?;
        self.send_haves(local_refs)?;
        // std::thread::sleep(std::time::Duration::from_secs(5));
        // self.wait_packfile()?;
        self.done()?;
        Ok(Some(hash))
    }

    /// Sends "have" lines to the remote Git repository, indicating the commits present locally.
    ///
    /// This method sends "have" lines to the remote Git repository, each specifying a commit hash that is present
    /// in the local Git repository. These lines help the remote repository identify commits it already has,
    /// reducing the data transfer during operations like "git fetch."
    ///
    /// # Arguments
    ///
    /// * `self`: A mutable reference to a `Client` instance, used to send "have" lines to the remote repository.
    /// * `local_refs`: A `HashMap` containing references (branch names) and their corresponding commit hashes
    ///   present in the local Git repository.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
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

    // Auxiliar function. Reads the socket until a 'flush' signal is read
    fn wait_packfile(&mut self) -> io::Result<()> {
        let socket = self.socket()?;
        while let Ok((size, bytes)) = read_pkt_line_bytes(socket) {
            if size < 4 {
                break;
            }
            if bytes[0] == 1 {
                let packfile = Packfile::reader(&bytes[..])?;
                for entry in packfile {
                    let entry = entry?;
                    hash_object::store_bytes_array_to_file(
                        entry.content,
                        &self.git_dir,
                        &entry.obj_type.to_string(),
                    )?;
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

// De aca abajo son funciones que sirven para el server tambien

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

/// Reads a Git "packet line" from the provided TCP socket and returns its size and content as bytes.
///
/// This function reads a Git "packet line" from the specified TCP socket. A Git "packet line" typically
/// consists of a 4-byte header indicating the line's total size, followed by the content. The function reads
/// and parses the header to determine the line's size and then reads the content as bytes. It returns both
/// the size and content as a tuple.
///
/// # Arguments
///
/// * `socket`: A mutable reference to the TCP socket from which to read the Git "packet line."
///
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

/// Retrieves the name of the currently checked-out branch from the Git repository's HEAD file.
///
/// This function reads the contents of the Git repository's "HEAD" file to determine the currently
/// checked-out branch and returns its name as a string. The "HEAD" file typically contains a reference
/// to the branch that is currently active.
///
/// # Arguments
///
/// * `git_dir`: A string representing the path to the Git repository's root directory.
///
/// # Returns
///
/// Returns a `Result` containing the name of the currently checked-out branch as a string. In case of success,
/// an `io::Result<String>` is returned.
///
fn get_head_branch(git_dir: &str) -> io::Result<String> {
    let head = PathBuf::from(git_dir).join("HEAD");
    let content = fs::read_to_string(head)?;
    let (_, branch) = content.rsplit_once("/").ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Invalid data HEAD. Must have ref for fetch",
    ))?;
    Ok(branch.trim().to_string())
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
            .split_once('/')
            .ok_or(Error::new(
                io::ErrorKind::Other,
                format!("Unknown error splitting path at '/': {:?}", path),
            ))?
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

/// Retrieves the set of missing Git objects between two commit hashes in a Git repository.
///
/// This function calculates and returns the set of missing Git objects (commits, trees, blobs, etc.)
/// between two commit hashes in a Git repository. It recursively traverses the commit history from
/// the new hash to the previous hash, identifying missing objects that need to be transferred during
/// operations like "git push."
///
/// # Arguments
///
/// * `new_hash`: A string representing the hash of the newer commit in the Git repository.
/// * `prev_hash`: A string representing the hash of the previous commit in the Git repository.
/// * `git_dir`: A string representing the path to the Git repository's root directory.
///
/// # Returns
///
/// Returns a `Result` containing a set of missing Git objects as tuples of `ObjectType` and string identifiers.
/// The `ObjectType` indicates the type of Git object, and the string identifier is its hash. In case of success,
/// an `io::Result<HashSet<(ObjectType, String)>>` is returned.
///
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

/// Retrieves Git objects (trees and blobs) referenced by a specific tree object in a Git repository.
///
/// This function reads a tree object's content and identifies the Git objects (trees and blobs) it references.
/// It recursively traverses nested trees, populating a set with tuples representing the object type and its hash.
///
/// # Arguments
///
/// * `hash`: A string representing the hash of the tree object in the Git repository.
/// * `git_dir`: A string representing the path to the Git repository's root directory.
///
/// # Returns
///
/// Returns a `Result` containing a set of Git objects as tuples of `ObjectType` and string identifiers.
/// The `ObjectType` indicates the type of Git object (Tree or Blob), and the string identifier is its hash.
/// In case of success, an `io::Result<HashSet<(ObjectType, String)>>` is returned.
///
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
