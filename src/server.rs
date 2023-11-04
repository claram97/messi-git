use crate::packfile_handler::create_packfile_from_set;
use crate::server_utils::*;

use std::collections::{HashSet, HashMap};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::{thread, fs};

const CAPABILITIES_UPLOAD: &str = "multi_ack side-band-64k ofs-delta";

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
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid command: {}", value))),
        }
    }
}

struct ServerInstace {
    socket: TcpStream,
    path: Arc<String>,
    commands: Vec<Command>,
    git_dir: String,
    git_dir_path: String,
}

impl ServerInstace {
    fn new(stream: TcpStream, path: Arc<String>, git_dir: &str) -> Self {
        let commands = vec![Command::UploadPack, Command::ReceivePack];
        Self { socket: stream, path: path.clone(), commands, git_dir: git_dir.to_string(), git_dir_path: String::default() }
    }

    fn handle_client(&mut self) -> io::Result<()> {
        let command = self.read_command()?;
        match command {
            Command::UploadPack => self.upload_pack()?,
            Command::ReceivePack => self.receive_pack()?,
        }
        Ok(())
    }

    fn read_command(&mut self) -> io::Result<Command> {
        let (_, command) = read_pkt_line(&mut self.socket)?;
        let (git_command, line) = command.split_once(" ").ok_or(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid command line: {}", command)))?;
        let (repo, _) = line.split_once("\0").ok_or(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid command line: {}", command)))?;
        self.git_dir_path = format!("{}{}/{}", self.path, repo, &self.git_dir);
        Command::try_from(git_command)
    }

    fn upload_pack(&mut self) -> io::Result<()> {
        self.send_refs()?;
        let wants = self.read_wants_haves(WantHave::Want)?;
        let haves = self.read_wants_haves(WantHave::Have)?;
        dbg!(&wants);
        dbg!(&haves);

        
        let (_, line) = read_pkt_line(&mut self.socket)?;
        if line != "done\n" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expecting done line: {}", line)));
        }
        
        let mut missing = HashSet::new();
        for want in wants {
            let m = get_missing_objects_from2(&want, &haves, &self.git_dir_path)?;
            missing.extend(m);
        }

        let packfile = create_packfile_from_set(missing, &self.git_dir_path)?;
        // self.send_bytes(&[1])?;
        // self.send_bytes(&packfile)?;
        let packfile: Vec<u8> = [vec![1], packfile].concat();
        self.send_bytes(&pkt_line_bytes(&packfile))?;

        Ok(())
    }

    fn read_wants_haves(&mut self, want_have: WantHave) -> io::Result<HashSet<String>> {
        let mut wants = HashSet::new();
        loop {
            let (size, line) = read_pkt_line(&mut self.socket)?;
            if size < 4 {
                break;
            }
            let hash = parse_line_want_have(&line, want_have)?;
            wants.insert(hash);
        }
        Ok(wants)
    }

    fn send_refs(&mut self) -> io::Result<()> {
        let mut refs = vec![];
        let server_refs_heads = get_head_refs(&self.git_dir_path)?;

        let head_path = PathBuf::from(&self.git_dir).join("HEAD");
        if head_path.exists() {
            let head_content = fs::read_to_string(head_path)?;
            if let Some((_, head)) = head_content.rsplit_once(": ") {
                let head = head.trim();
                if let Some(hash) = server_refs_heads.get(head) {
                    refs.push(format!("{} {}", hash, head));
                }
            }
        }

        refs.extend(server_refs_heads.iter().map(|(k, v)| format!("{} refs/heads/{}", v, k)));
        refs[0] = format!("{}\0{}", refs[0], CAPABILITIES_UPLOAD);

        for r in refs {
            self.send(&pkt_line(&r))?;
        }
        
        self.flush()
    }

    fn receive_pack(&mut self) -> io::Result<()> {
        dbg!("Llego a receive pack");
        Ok(())
    }

    // fn end_connection(&mut self) -> io::Result<()> {
    //     self.flush()?;
    //     Ok(())
    // }

    // Sends a message throw the socket
    fn send(&mut self, message: &str) -> io::Result<()> {
        dbg!(message);
        write!(self.socket, "{}", message)
    }

    // Sends a message throw the socket as bytes
    fn send_bytes(&mut self, content: &[u8]) -> io::Result<()> {
        dbg!("Sending bytes...");
        self.socket.write_all(content)
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

// git_dir: .mgit, .git ... git dir name
pub fn run(domain: &str, port: &str, path: &str, git_dir: &str) -> io::Result<()> {
    let address = domain.to_owned() + ":" + port;
    let listener = TcpListener::bind(address)?;
    let path = Arc::new(String::from(path));

    let mut handles = vec![];
    while let Ok((client_stream, _socket_addr)) = listener.accept() {
        let dir = git_dir.to_string();
        let path_clone = path.clone();
        let handle = thread::spawn(move || {
            let mut server = ServerInstace::new(client_stream, path_clone, &dir);
            dbg!(server.handle_client())
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}