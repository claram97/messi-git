use std::{
    collections::HashMap,
    fs,
    io::{self, Error, Read, Write},
    net::TcpStream,
    path::PathBuf,
    str::from_utf8,
    vec,
};

// multi_ack_detailed side-band-64k thin-pack include-tag ofs-delta deepen-since deepen-not

const PORT: &str = "9418";
const VERSION: &str = "1";
const GIT_UPLOAD_PACK: &str = "git-upload-pack";
const CAPABILITIES: &str = "multi_ack_detailed side-band-64k wait-for-done";
#[derive(Debug)]
struct Client {
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
        self.initiate_connection()?;
        self.wait_refs()?;
        self.flush()?;
        Ok(self.refs.keys().map(String::from).collect())
    }

    // REVISAR: deberia ser como el upload-pack the git
    pub fn upload_pack(&mut self, wanted_ref: Option<&str>, git_dir: &str) -> io::Result<()> {
        self.connect()?;
        self.initiate_connection()?;
        self.git_dir = PathBuf::from(git_dir);
        self.wait_refs()?;

        if let Some(wanted_ref) = wanted_ref {
            if let Some(hash) = self.want_ref(wanted_ref)? {
                self.update_fetch_head(&hash)?;
                println!("Leo response de want");
                self.read_response_until_flush()?;
            }
        }
        // std::thread::sleep(std::time::Duration::from_secs(5));

        println!("Termino conexion");
        self.flush()
    }

    // Establish the first conversation with the server
    // Lets the server know that an upload-pack is requested
    fn initiate_connection(&mut self) -> io::Result<()> {
        let mut command = format!("{} /{}", GIT_UPLOAD_PACK, self.repository);

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
        let (_, _) = read_pkt_line_tcp(self.socket()?);
        let (mut size, mut line) = read_pkt_line_tcp(self.socket()?);

        while size > 0 {
            if let Some((hash, mut ref_path)) = line.split_once(" ") {
                if let Some(("HEAD", _capabilities)) = ref_path.split_once("\0") {
                    // self.capabilities = capabilities.to_string();
                    ref_path = "HEAD";
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
            None => return Err(connection_not_established_error()),
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
        // for wanted_ref in wanted_refs {
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

        let want = format!("want {} {}\n", hash, CAPABILITIES);
        let want = pkt_line(&want);
        dbg!(&want);
        self.send(&want)?;
        // despues de todos los want -> flush
        self.flush()?;
        // despues de flush -> los have
        // let have = pkt_line("have fd443c581db78b7f422f5eb4052aef10af1c01b5\n");
        // self.send(have)?;
        // despues de todos los have -> flush
        // self.send(PKT_FLUSH)?;
        // despues de todo -> DONE
        self.done()?;
        Ok(Some(String::from(hash)))

        // }
        // Ok(())
    }

    // Auxiliar function. Reads the socket until a 'flush' signal is read
    fn read_response_until_flush(&mut self) -> io::Result<()> {
        // let mut reader = BufReader::new(&self.socket);
        // let (mut size, mut line) = read_pkt_line(&mut reader);
        // while size > 0 {
        //     print!("{}", line);
        //     (size, line) = read_pkt_line(&mut reader);
        // }

        let mut socket = self.socket()?;

        let (mut size, mut line) = read_pkt_line_tcp(&mut socket);

        while size > 0 {
            print!("{}", line);
            (size, line) = read_pkt_line_tcp(&mut socket);
        }
        println!();
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
    let size = match usize::from_str_radix(&size, 16) {
        Ok(n) => n,
        Err(_) => 0,
    };

    if size < 4 {
        return (size, String::new());
    }
    let line = read_n_to_string_tcp(socket, size - 4);
    (size, line)
}

fn read_n_to_string_tcp(socket: &mut TcpStream, n: usize) -> String {
    let mut buf = vec![0u8; n];
    match (socket.read_exact(&mut buf), from_utf8(&buf)) {
        (Ok(_), Ok(content)) => content.to_owned(),
        _ => String::new(),
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

fn get_local_refs(git_dir: &PathBuf) -> io::Result<HashMap<String, String>> {
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
            .split_once("/")
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

#[cfg(test)]
mod tests {
    use super::*;
    // Command to raise a git server with git daemon is a directory with git directories inside.
    // git-daemon-export-ok file should exist in .git directory in the repo
    // git daemon --base-path=. --reuseaddr --informative-errors --verbose
    #[test]
    #[ignore]
    fn test_get_refs() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::new(&address, "repo", "localhost")?;
        assert!(!client.get_refs()?.is_empty());
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_get_refs2() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::new(&address, "repo", "localhost")?;
        client.get_refs()?;
        client.get_refs()?;
        assert!(!client.get_refs()?.is_empty());
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_refs_has_head() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::new(&address, "repo", "localhost")?;
        let refs = client.get_refs()?;
        assert!(refs.contains(&"HEAD".to_string()));
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_upload_pack() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::new(&address, "repo", "localhost")?;
        client.upload_pack(Some("HEAD"), ".mgit")?;
        Ok(())
    }
}
