use std::{
    collections::HashMap,
    fs,
    io::{self, BufReader, Error, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    str::from_utf8,
    vec,
};

use crate::packfile_handler::Packfile;

// multi_ack_detailed side-band-64k thin-pack include-tag ofs-delta deepen-since deepen-not

const VERSION: &str = "1";
const GIT_UPLOAD_PACK: &str = "git-upload-pack";
const GIT_RECEIVE_PACK: &str = "git-receive-pack";
const CAPABILITIES_UPLOAD: &str = "multi_ack side-band-64k ofs-delta";
#[derive(Debug)]
pub struct Client {
    git_dir: PathBuf,
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
            git_dir: PathBuf::new(),
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
        self.git_dir = PathBuf::from(git_dir);
        self.wait_refs()?;

        if let Some(wanted_ref) = wanted_ref {
            if let Some(hash) = self.want_ref(wanted_ref)? {
                self.update_fetch_head(&hash)?;
                self.wait_packfile()?;
            }
        }
        println!("Termino upload-pack");
        Ok(())

        // self.flush()
    }

    pub fn receive_pack(&mut self, pushing_ref: &str, git_dir: &str) -> io::Result<()> {
        self.connect()?;
        self.initiate_connection(GIT_RECEIVE_PACK)?;
        self.flush()?;
        self.git_dir = PathBuf::from(git_dir);

        // ya se que tiene el servidor
        self.wait_refs()?;

        let local_refs = get_local_refs(&self.git_dir)?;
        let new_hash = match local_refs.get(pushing_ref) {
            Some(hash) => hash,
            None => return Err(Error::new(io::ErrorKind::NotFound, "Ref not found")),
        };

        let prev_hash = match self.refs.get(pushing_ref) {
            Some(hash) => hash.clone(),
            None => String::new(),
        };

        if prev_hash.is_empty() {
            self.receive_pack_create(pushing_ref, new_hash)
        } else {
            self.receive_pack_update(pushing_ref, &prev_hash, new_hash)
        }
    }

    fn receive_pack_create(&mut self, pushing_ref: &str, hash: &str) -> io::Result<()> {
        Ok(())
    }
    fn receive_pack_update(
        &mut self,
        pushing_ref: &str,
        prev_hash: &str,
        new_hash: &str,
    ) -> io::Result<()> {
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
        let (_, _) = read_pkt_line_tcp(self.socket()?);
        let (mut size, mut line) = read_pkt_line_tcp(self.socket()?);

        while size > 0 {
            if let Some((hash, mut ref_path)) = line.split_once(' ') {
                if let Some((head, _capabilities)) = ref_path.split_once('\0') {
                    // self.capabilities = capabilities.to_string();
                    ref_path = head;
                }
                self.refs
                    .insert(ref_path.trim().to_string(), hash.to_string());
            }
            (size, line) = read_pkt_line_tcp(self.socket()?);
        }
        dbg!(&self.refs);
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

    fn update_fetch_head(&self, hash: &str) -> io::Result<()> {
        let fetch_head = self.git_dir.join("FETCH_HEAD");
        let mut fetch_head = fs::File::create(fetch_head)?;
        writeln!(fetch_head, "{}", hash)
    }

    // Tells the server which refs are required
    // If the provided ref name does not exist in remote, then an error is returned.
    //
    // (DEBERIA TRAER EL PACKFILE PERO TODAVIA NO LO HACE)
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

        let local_refs = get_local_refs(&self.git_dir)?;

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

        let mut buf: [u8; 4] = [0, 0, 0, 0];
        let mut reader = BufReader::new(socket);

        reader.read_exact(&mut buf)?;

        while buf != "0000".as_bytes() {
            let bytes = from_utf8(&buf).unwrap_or_default();
            print!("{}", bytes);
            let bytes = usize::from_str_radix(bytes, 16).unwrap_or_default();

            if bytes < 4 {
                break;
            }

            let mut content = vec![0; bytes - 4];
            reader.read_exact(&mut content)?;

            let is_header_start = content[0] == 1;

            if is_header_start {
                let packfile = Packfile::new(&content[..])?;
                for obj in packfile {
                    println!("{}", obj)
                }
            } else {
                let content = from_utf8(&content)
                    .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
                print!("{}", content);
            };

            reader.read_exact(&mut buf)?;
        }

        Ok(())
    }
}

fn connection_not_established_error() -> Error {
    Error::new(
        io::ErrorKind::BrokenPipe,
        "The operation failed because the connection was not established.",
    )
}

fn read_pkt_line_tcp(socket: &mut TcpStream) -> (usize, String) {
    let size = read_n_to_string_tcp(socket, 4);
    let size = usize::from_str_radix(&size, 16).unwrap_or(0);

    if size < 4 {
        return (size, String::new());
    }
    let line = read_n_to_string_tcp(socket, size - 4);
    (size, line)
}

fn read_n_to_string_tcp(socket: &mut TcpStream, n: usize) -> String {
    let mut buf = vec![0u8; n];

    match socket.read_exact(&mut buf) {
        Ok(_) => match from_utf8(&buf) {
            Ok(content) => content.to_owned(),
            Err(_) => {
                todo!()
            }
        },
        Err(_) => todo!(),
    }
}

fn pkt_line(line: &str) -> String {
    let len = line.len() + 4; // len
    let mut len_hex = format!("{len:x}");
    while len_hex.len() < 4 {
        len_hex = "0".to_owned() + &len_hex
    }
    len_hex + line
}

fn get_local_refs(git_dir: &Path) -> io::Result<HashMap<String, String>> {
    let mut refs = HashMap::new();
    let heads = git_dir.join("refs").join("heads");
    for entry in fs::read_dir(&heads)? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();
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

    let head = git_dir.join("HEAD");

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
    dbg!(&refs);
    Ok(refs)
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.end_connection();
    }
}
