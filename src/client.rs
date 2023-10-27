use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    str::from_utf8,
    vec,
};

// multi_ack_detailed side-band-64k thin-pack include-tag ofs-delta deepen-since deepen-not

const PORT: &str = "9418";
const VERSION: &str = "1";
const GIT_UPLOAD_PACK: &str = "git-upload-pack";

#[derive(Debug)]
struct Client {
    repository: String,
    host: String,
    socket: TcpStream,
    refs: HashMap<String, String>,
    capabilities: String,
}

/// This is a git client that is able to connect to a git server
/// using the git protocol.
impl Client {

    /// Establish a connection with the server and asks for the refs in the remote.
    /// A hashmap with the path of the refs as keys and the last commit hash as values is returned.
    /// 
    /// May fail due to I/O errors
    pub fn get_refs(&mut self) -> io::Result<&HashMap<String, String>> {
        self.upload_pack_initiate_connection()?;
        self.upload_pack_wait_refs();
        self.flush()?;
        Ok(&self.refs)
    }

    /// Creates a connection with a server (assuming its a git server)
    /// 
    /// Parameters:
    ///     - address: address to establish a tcp connection
    ///     - repository: name of the repository in the remote
    ///     - host: REVISAR (no se si es si o si el mismo que address)
    pub fn connect(address: &str, repository: &str, host: &str) -> io::Result<Self> {
        let socket = TcpStream::connect(address)?;
        let refs = HashMap::new();
        // let capabilities = String::new();
        let repository = repository.to_owned();
        let host = host.to_owned();
        let capabilities = String::from("multi_ack_detailed side-band-64k wait-for-done");
        Ok(Self {
            repository,
            host,
            socket,
            refs,
            capabilities,
        })
    }

    // Sends a message throw the socket
    fn send(&mut self, message: &str) -> io::Result<()> {
        write!(self.socket, "{}", message)
    }

    // Sends a 'flush' signal to the server
    fn flush(&mut self) -> io::Result<()> {
        self.send("0000")
    }

    // Sends a 'done' signal to the server
    fn done(&mut self) -> io::Result<()> {
        self.send("0009done\n")
    }

    /// REVISAR: deberia ser como el upload-pack the git
    pub fn upload_pack(&mut self) -> io::Result<()> {
        self.upload_pack_initiate_connection()?;
        // self.ls_refs()?;
        // self.read_response_until_flush();
        self.upload_pack_wait_refs();

        self.upload_pack_send_wanted_refs()?;
        println!("Leo response de want");
        // std::thread::sleep(std::time::Duration::from_secs(5));
        self.read_response_until_flush();

        println!("Termino conexion");
        self.flush()
    }

    // Selects the wanted refs to fetch 
    fn select_wanted_refs(&self) -> Vec<String> {
        vec!["HEAD".to_string()]
    }

    // Auxiliar function. Tells the server which refs are required
    fn upload_pack_send_wanted_refs(&mut self) -> io::Result<()> {
        let _wanted_refs = self.select_wanted_refs();
        let wanted_ref = "refs/heads/sharing";
        println!("Pido: {}", wanted_ref);
        // for wanted_ref in wanted_refs {
        let hash = match self.refs.get(wanted_ref) {
            Some(hash) => hash,
            None => return Ok(()),
        };

        let want = format!("want {} {}\n", hash, &self.capabilities);
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
        self.done()

        // }
        // Ok(())
    }

    // Establish the first conversation with the server
    // Lets the server know that an upload-pack is requested
    fn upload_pack_initiate_connection(
        &mut self
    ) -> io::Result<()> {
        let mut command = format!("{} /{}", GIT_UPLOAD_PACK, self.repository);

        command = format!("{}\0host={}\0", command, self.host);

        command = format!("{}\0version={}\0", command, VERSION);

        let pkt_command = pkt_line(&command);

        println!("Enviando al socket: {:?}", &pkt_command);

        self.send(&pkt_command)?;
        println!("Termino de enviar al socket");
        Ok(())
    }

    // Auxiliar function. Reads the socket until a 'flush' signal is read
    fn read_response_until_flush(&mut self) {
        let mut reader = BufReader::new(&self.socket);
        let (mut size, mut line) = read_pkt_line(&mut reader);
        while size > 0 {
            print!("{}", line);
            (size, line) = read_pkt_line(&mut reader);
        }
        println!();
    }

    // Auxiliar function. Waits for the refs and loads them in self
    // Should be called only aftes: upload_pack_initiate_connection 
    fn upload_pack_wait_refs(&mut self) {
        let mut reader = BufReader::new(&self.socket);
        let _ = reader.read_line(&mut String::new());

        let (mut size, mut line) = read_pkt_line(&mut reader);

        while size > 0 {
            if let Some((hash, mut ref_path)) = line.split_once(" ") {
                if let Some(("HEAD", _capabilities)) = ref_path.split_once("\0") {
                    // self.capabilities = capabilities.to_string();
                    ref_path = "HEAD";
                }
                self.refs
                    .insert(ref_path.trim().to_string(), hash.to_string());
            }
            (size, line) = read_pkt_line(&mut reader);
        }
        dbg!(&self.refs);
    }

}


fn read_pkt_line(buf_reader: &mut BufReader<impl Read>) -> (usize, String) {
    let size = read_n_to_string(buf_reader, 4);
    let size = match usize::from_str_radix(&size, 16) {
        Ok(n) => n,
        Err(_) => 0,
    };

    if size < 4 {
        return (size, String::new());
    }
    let line = read_n_to_string(buf_reader, size - 4);
    (size, line)
}

fn read_n_to_string(buf_reader: &mut BufReader<impl Read>, n: usize) -> String {
    let mut buf = vec![0u8; n];
    match (buf_reader.read_exact(&mut buf), from_utf8(&buf)) {
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
        let mut client = Client::connect(&address, "repo","localhost")?;
        assert!(!client.get_refs()?.is_empty());
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_refs_has_head() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::connect(&address, "repo", "localhost")?;
        assert!(client.get_refs()?.contains_key("HEAD"));
        Ok(())
    }
}