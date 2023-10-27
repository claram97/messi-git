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
const HOST: &str = "localhost";
#[derive(Debug)]
struct Client {
    socket: TcpStream,
    refs: HashMap<String, String>,
    capabilities: String,
}

impl Client {

    fn get_refs(&self) -> &HashMap<String, String> {
        &self.refs
    }

    fn connect(address: &str) -> io::Result<Self> {
        let socket = TcpStream::connect(address)?;
        let refs = HashMap::new();
        // let capabilities = String::new();
        let capabilities = String::from("multi_ack_detailed side-band-64k wait-for-done");
        Ok(Self {
            socket,
            refs,
            capabilities,
        })
    }

    fn send(&mut self, message: &str) -> io::Result<()> {
        write!(self.socket, "{}", message)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.send("0000")
    }

    fn done(&mut self) -> io::Result<()> {
        self.send("0009done\n")
    }

    pub fn upload_pack(&mut self, repository_name: &str, host: Option<&str>) -> io::Result<()> {
        self.upload_pack_initiate_connection(repository_name, host)?;
        // self.ls_refs()?;
        // self.read_response_until_flush();
        self.upload_pack_wait_refs();
        dbg!(&self.refs);

        self.upload_pack_send_wanted_refs()?;
        println!("Leo response de want");
        // std::thread::sleep(std::time::Duration::from_secs(5));
        self.read_response_until_flush();

        println!("Termino conexion");
        self.end_connection()
    }

    fn select_wanted_refs(&self) -> Vec<String> {
        vec!["HEAD".to_string()]
    }

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

    fn upload_pack_initiate_connection(
        &mut self,
        repository_name: &str,
        _host: Option<&str>,
    ) -> io::Result<()> {
        let mut command = format!("{} /{}", GIT_UPLOAD_PACK, repository_name);
        // if let Some(host) = host {
        command = format!("{}\0host={}\0", command, HOST);
        // }
        command = format!("{}\0version={}\0", command, VERSION);

        let pkt_command = pkt_line(&command);

        println!("Enviando al socket: {:?}", &pkt_command);

        self.send(&pkt_command)?;
        println!("Termino de enviar al socket");
        Ok(())
    }

    fn read_response_until_flush(&mut self) {
        let mut reader = BufReader::new(&self.socket);
        let (mut size, mut line) = read_pkt_line(&mut reader);
        while size > 0 {
            print!("{}", line);
            (size, line) = read_pkt_line(&mut reader);
        }
        println!();
    }

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
    }

    fn end_connection(&mut self) -> io::Result<()> {
        self.flush()
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

    #[test]
    #[ignore]
    fn test_get_refs() -> io::Result<()> {
        let address = "localhost:".to_owned() + PORT;
        let mut client = Client::connect(&address)?;
        client.upload_pack("repo", None)?;
        assert!(!client.get_refs().is_empty());
        Ok(())
    }
}