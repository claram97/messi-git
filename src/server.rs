use crate::packfile::handler::{create_packfile, unpack_packfile};
use crate::server_utils::*;

use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};

const CAPABILITIES: &str = "multi_ack side-band-64k ofs-delta";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000";

enum Command {
    UploadPack,
    ReceivePack,
}

impl TryFrom<&str> for Command {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "git-upload-pack" => Ok(Command::UploadPack),
            "git-receive-pack" => Ok(Command::ReceivePack),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid command: {}", value),
            )),
        }
    }
}

struct ServerInstace {
    socket: TcpStream,
    path: String,
    git_dir: String,
    git_dir_path: String,
}

impl ServerInstace {
    // Creates a new instance of the server changing the current dir where the repositories are stored
    fn new(stream: TcpStream, path: Arc<String>, git_dir: &str) -> io::Result<Self> {
        env::set_current_dir(path.clone().as_ref())?;
        log(&format!("New connection from {}", stream.peer_addr()?))?;
        Ok(Self {
            socket: stream,
            path: path.to_string(),
            git_dir: git_dir.to_string(),
            git_dir_path: String::default(),
        })
    }

    // Handles the client requests
    fn handle_client(&mut self) -> io::Result<()> {
        let command = match self.read_command() {
            Ok(command) => command,
            Err(e) => {
                self.send(&pkt_line(&format!("ERR {}\n", e)))?;
                return Err(e);
            }
        };
        let result = match command {
            Command::UploadPack => self.upload_pack(),
            Command::ReceivePack => self.receive_pack(),
        };
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                self.send(&pkt_line(&format!("ERR {}\n", e)))?;
                Err(e)
            }
        }
    }

    // Reads the command sent by the client
    fn read_command(&mut self) -> io::Result<Command> {
        let (_, command) = read_pkt_line(&mut self.socket)?;
        let (git_command, line) = command.split_once(' ').ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid command line: {}", command),
        ))?;
        let (repo, _) = line.split_once('\0').ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid command line: {}", command),
        ))?;

        self.git_dir_path = format!("{}{}/{}", self.path, repo, &self.git_dir);
        if !Path::new(&self.git_dir_path).exists() {
            self.git_dir_path = format!("{}{}", self.path, repo);
        }
        log(&format!("Command: {}", git_command))?;
        Command::try_from(git_command)
    }

    // Sends the refs to the client
    // Receiving the wants and haves from the client used to calculate the missing objects
    // Then, the packfile is created and sent to the client
    fn upload_pack(&mut self) -> io::Result<()> {
        self.send_refs()?;
        let (wants, haves) = self.read_wants_haves()?;
        if wants.is_empty() {
            log("No wants")?;
            return Ok(());
        }
        let mut missing = HashSet::new();
        for want in wants {
            let m = get_missing_objects_from(&want, &haves, &self.git_dir_path)?;
            missing.extend(m);
        }
        let mut missing = missing.into_iter().collect::<Vec<String>>();
        missing.sort();
        log(&format!("Missing: {:?}", missing))?;
        let packfile = create_packfile(&missing, &self.git_dir_path)?;
        let packfile: Vec<u8> = [vec![1], packfile].concat();
        self.send_bytes(&pkt_line_bytes(&packfile))
    }

    // Receives the packfile from the client
    // After receiving it, it is unpacked and stored in the git_dir
    // Then, the refs are updated
    fn receive_pack(&mut self) -> io::Result<()> {
        self.send_refs()?;
        let new_refs = self.wait_changes()?;

        if new_refs.is_empty() {
            return Ok(());
        }

        let wait_for_packfile = new_refs.iter().any(|(_, (_, new))| new != ZERO_HASH);
        if wait_for_packfile {
            self.wait_and_unpack_packfile()?;
        };
        self.make_refs_changes(new_refs)
    }

    // Sends the server refs to the client
    fn send_refs(&mut self) -> io::Result<()> {
        log("Sending refs...")?;
        let mut refs = vec![];
        let server_refs_heads = get_head_tags_refs(&self.git_dir_path)?;

        let head_path = PathBuf::from(&self.git_dir_path).join("HEAD");
        if head_path.exists() {
            let head_content = read_file_with_lock(&head_path)?;
            if let Some((_, head)) = head_content.rsplit_once('/') {
                let head = head.trim();
                if let Some(hash) = server_refs_heads.get(head) {
                    refs.push(format!("{} {}", hash, "HEAD"));
                }
            }
        }

        refs.extend(
            server_refs_heads
                .iter()
                .map(|(k, v)| format!("{} refs/heads/{}", v, k)),
        );

        if refs.is_empty() {
            let empty = format!("{} {}\0{}", ZERO_HASH, "capabilities^{}", CAPABILITIES);
            self.send(&pkt_line(&empty))?;
            return self.flush();
        }

        refs[0] = format!("{}\0{}", refs[0], CAPABILITIES);

        let version = "version 1";
        let version = pkt_line(version);
        self.send(&version)?;

        for r in refs {
            self.send(&pkt_line(&r))?;
        }

        self.flush()
    }

    // Reads the wants or haves sent by the client
    // Returns a set of hashes of wants or haves, depending on the parameter
    fn read_wants_haves(&mut self) -> io::Result<(HashSet<String>, HashSet<String>)> {
        let mut wants = HashSet::new();
        let mut haves = HashSet::new();
        let mut total_read = 0;
        loop {
            let (size, line) = read_pkt_line(&mut self.socket)?;
            total_read += size;
            if total_read == 0 || line == "done\n" {
                break;
            }
            if size < 4 {
                continue;
            }
            let (t, hash) = parse_line_want_have(&line)?;
            match t {
                WantHave::Want => wants.insert(hash),
                WantHave::Have => haves.insert(hash),
            };
        }
        log(&format!("Wants: {:?}. Haves: {:?}", wants, haves))?;
        Ok((wants, haves))
    }

    // Waits for the client to send a packfile
    // After receiving it, it is unpacked and stored in the git_dir
    fn wait_and_unpack_packfile(&mut self) -> io::Result<()> {
        log("Waiting for packfile...")?;
        loop {
            let (size, bytes) = read_pkt_line_bytes(&mut self.socket)?;
            if size < 4 {
                break;
            }
            if bytes[0] == 1 {
                return unpack_packfile(&bytes[..], &self.git_dir_path);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Packfile not found",
        ))
    }

    // Updates the refs with the new ones received from the client
    fn make_refs_changes(&mut self, new_refs: HashMap<String, (String, String)>) -> io::Result<()> {
        log("Updating refs...")?;
        for (ref_name, (old, new)) in &new_refs {
            match (old, new) {
                (old, new) if old == ZERO_HASH => self.create_ref(ref_name, new)?,
                (_old, new) if new == ZERO_HASH => self.delete_ref(ref_name)?,
                (old, new) => self.update_ref(ref_name, old, new)?,
            }
        }
        Ok(())
    }

    // Creates a new ref with the given name and hash
    // The ref must not exist
    fn create_ref(&mut self, ref_name: &str, new: &str) -> io::Result<()> {
        log(&format!("Creating ref: {} -> {}", ref_name, new))?;
        let ref_path = PathBuf::from(&self.git_dir_path).join(ref_name);
        if ref_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Ref already exists: {}. Use update", ref_name),
            ));
        }
        let content = [new.as_bytes(), b"\n"].concat();
        write_file_with_lock(ref_path, content)?;
        Ok(())
    }

    // Updates a ref with the given name and hash
    // The old hash must be the same as the one stored in the ref
    // The ref must exist
    fn update_ref(&mut self, ref_name: &str, old: &str, new: &str) -> io::Result<()> {
        log(&format!("Updating ref: {} -> {}", ref_name, new))?;
        let ref_path = PathBuf::from(&self.git_dir_path).join(ref_name);
        if !ref_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Ref not found: {}. Can not update", ref_name),
            ));
        }

        if read_file_with_lock(&ref_path)?.trim() != old {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Ref is not at expected hash: {}. Can not update", ref_name),
            ));
        }
        let content = [new.as_bytes(), b"\n"].concat();
        write_file_with_lock(ref_path, content)?;
        Ok(())
    }

    // Deletes a ref with the given name
    fn delete_ref(&mut self, ref_name: &str) -> io::Result<()> {
        log(&format!("Deleting ref: {}", ref_name))?;
        let ref_path = PathBuf::from(&self.git_dir).join(ref_name);
        fs::remove_file(ref_path)
    }

    // Waits for the client to send the new refs
    // Returns a hashmap with the new refs and the old and new hashes
    // Will fail if the client tries to update the actual branch (same as git daemon)
    fn wait_changes(&mut self) -> io::Result<HashMap<String, (String, String)>> {
        let mut new_refs = HashMap::new();
        loop {
            let (size, line) = read_pkt_line(&mut self.socket)?;
            if size < 4 {
                break;
            }
            let (old, new, ref_name) = {
                let (old, line) = line.split_once(' ').ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid line: {}", line),
                ))?;
                let (new, line) = line.split_once(' ').ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid line: {}", line),
                ))?;
                let (ref_name, _) = line.split_once('\0').ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid line: {}", line),
                ))?;
                (
                    old.to_string(),
                    new.to_string(),
                    ref_name.trim().to_string(),
                )
            };

            let head_ref = match get_head_from_branch(&self.git_dir_path, "HEAD") {
                Ok(head) => head,
                Err(e) if e.kind() == io::ErrorKind::InvalidData => String::new(),
                Err(e) => return Err(e),
            };
            if !is_repo_bare(&self.git_dir_path)? && ref_name == head_ref {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Can not update actual branch. Please do a checkout and try again",
                ));
            }
            new_refs.insert(ref_name, (old, new));
        }
        log(&format!("New refs: {:?}", new_refs))?;
        Ok(new_refs)
    }

    // Sends a message through the socket
    fn send(&mut self, message: &str) -> io::Result<()> {
        log(&format!("Sending: {}", message))?;
        write!(self.socket, "{}", message)
    }

    // Sends a message through the socket as bytes
    fn send_bytes(&mut self, content: &[u8]) -> io::Result<()> {
        log(&format!("Sending bytes: {:?}", content))?;
        self.socket.write_all(content)
    }

    // Sends a 'flush' signal to the client
    fn flush(&mut self) -> io::Result<()> {
        self.send("0000")
    }
}

/// Runs a git server
///
/// # Arguments
///
/// * `domain` - The domain where the server will be listening
/// * `port` - The port where the server will be listening
/// * `path` - The path where the repositories are stored
/// * `git_dir` - The name of the directory where the git files are stored
pub fn run(domain: &str, port: &str, path: &str, git_dir: &str) -> io::Result<()> {
    let address = domain.to_owned() + ":" + port;
    let listener = TcpListener::bind(address)?;
    let path = Arc::new(String::from(path));

    let mut handles = vec![];
    while let Ok((client_stream, _socket_addr)) = listener.accept() {
        let dir = git_dir.to_string();
        let path_clone = path.clone();
        let handle = thread::spawn(move || {
            let mut server = ServerInstace::new(client_stream, path_clone, &dir)?;
            match server.handle_client() {
                Ok(_) => Ok(()),
                Err(err) => {
                    println!("Error: {}", err);
                    Err(err)
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}

/// Reads the contents of a file with an exclusive lock.
///
/// This function opens the specified file, acquires a lock, and reads the entire
/// contents of the file into a `String`. The lock ensures exclusive access to the file
/// during the reading process, preventing concurrent modifications.
///
/// # Arguments
///
/// * `path` - A reference to the path of the file to be read.
///
/// # Returns
///
/// An `io::Result` containing the file's contents as a `String` if successful, or an
/// `io::Error` if an error occurs.
///
/// # Errors
///
/// This function returns an `io::Error` if it encounters any issues during file opening,
/// lock acquisition, or content reading.
///
fn read_file_with_lock<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(path)?;
    let file_mutex = Mutex::new(file);
    let result = match file_mutex.lock() {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
        Err(e) => Err(io::Error::new(io::ErrorKind::WouldBlock, e.to_string())),
    };
    result
}

/// Writes the specified contents to a file with an exclusive lock.
///
/// This function creates or truncates the specified file, acquires a lock, and writes
/// the provided contents to the file. The lock ensures exclusive access to the file
/// during the writing process, preventing concurrent modifications.
///
/// # Arguments
///
/// * `path` - A reference to the path of the file to be written.
/// * `contents` - A reference to the contents to be written to the file.
///
/// # Returns
///
/// An `io::Result` containing the number of bytes written if successful, or an
/// `io::Error` if an error occurs.
///
/// # Errors
///
/// This function returns an `io::Error` if it encounters any issues during file creation,
/// lock acquisition, or content writing.
///
fn write_file_with_lock<P, C>(path: P, contents: C) -> io::Result<usize>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let file = fs::File::create(path)?;
    let file_mutex = Mutex::new(file);
    let size = match file_mutex.lock() {
        Ok(mut file) => file.write(contents.as_ref()),
        Err(e) => Err(io::Error::new(io::ErrorKind::WouldBlock, e.to_string())),
    };
    size
}

/// Checks whether a Git repository is bare or not.
///
/// This function examines the Git configuration file to determine if the repository
/// is bare. A repository is considered bare if it doesn't have a `config` file.
/// If the `config` file exists, it checks the content for the presence of the `bare` flag.
///
/// # Arguments
///
/// * `git_dir` - A string representing the path to the Git repository directory.
///
/// # Returns
///
/// An `io::Result` containing a boolean indicating whether the repository is bare (`true`)
/// or not (`false`). Returns an `io::Error` if there are issues accessing or reading
/// the repository configuration.
///
/// # Errors
///
/// This function returns an `io::Error` if it encounters any issues accessing or reading
/// the repository configuration file.
///
fn is_repo_bare(git_dir: &str) -> io::Result<bool> {
    let config_path = PathBuf::from(git_dir).join("config");
    if !config_path.exists() {
        return Ok(true);
    }
    let content = read_file_with_lock(config_path)?;
    let bare = content
        .lines()
        .find(|l| l.trim().starts_with("bare"))
        .ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Can not find bare config",
        ))?;
    let (_, bare) = bare.split_once('=').ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Can not find bare config",
    ))?;
    let bare = bare.trim();
    Ok(bare == "true")
}
