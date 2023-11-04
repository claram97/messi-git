use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Error, Read},
    net::TcpStream,
    path::PathBuf,
    str::from_utf8,
};

use crate::{
    cat_file, hash_object,
    packfile_handler::{ObjectType, Packfile},
};

pub fn connection_not_established_error() -> Error {
    Error::new(
        io::ErrorKind::BrokenPipe,
        "The operation failed because the connection was not established.",
    )
}

pub fn unpack_packfile(packfile: &[u8], git_dir: &str) -> io::Result<()> {
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
pub fn read_pkt_line(socket: &mut TcpStream) -> io::Result<(usize, String)> {
    let (size, bytes) = read_pkt_line_bytes(socket)?;
    let line = from_utf8(&bytes).unwrap_or_default().to_string();
    Ok((size, line))
}

pub fn read_pkt_line_bytes(socket: &mut TcpStream) -> io::Result<(usize, Vec<u8>)> {
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
pub fn pkt_line(line: &str) -> String {
    let len = line.len() + 4; // len
    let mut len_hex = format!("{len:x}");
    while len_hex.len() < 4 {
        len_hex = "0".to_owned() + &len_hex
    }
    len_hex + line
}

// Given some bytes to send a git client/server, this function transform it
// in PKT format
pub fn pkt_line_bytes(content: &[u8]) -> Vec<u8> {
    let len = content.len() + 4; // len
    let mut len_hex = format!("{len:x}");
    while len_hex.len() < 4 {
        len_hex = "0".to_owned() + &len_hex
    }
    let mut pkt_line = len_hex.as_bytes().to_vec();
    pkt_line.extend(content);
    pkt_line
}

pub fn get_head_from_branch(git_dir: &str, branch: &str) -> io::Result<String> {
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
pub fn get_head_refs(git_dir: &str) -> io::Result<HashMap<String, String>> {
    let pathbuf = PathBuf::from(git_dir);
    let heads = pathbuf.join("refs").join("heads");
    get_refs(heads)
}

// Auxiliar function which get refs under refs/heads
pub fn get_remote_refs(git_dir: &str, remote: &str) -> io::Result<HashMap<String, String>> {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WantHave {
    Want,
    Have,
}

impl TryFrom<&str> for WantHave {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "want" => Ok(Self::Want),
            "have" => Ok(Self::Have),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid want/have: {}", value),
            )),
        }
    }
}

pub fn parse_line_want_have(line: &str, want_have: WantHave) -> io::Result<String> {
    let (want_or_have, hash) = line.split_once(" ").ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Invalid want line: {}", line),
    ))?;

    if WantHave::try_from(want_or_have)? != want_have {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expecting want line: {}", line),
        ));
    }

    let (hash, _) = hash.split_once(" ").unwrap_or((hash, ""));
    Ok(hash.trim().to_string())
}

pub fn get_missing_objects_from(
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

pub fn get_missing_objects_from2(
    want: &str,
    haves: &HashSet<String>,
    git_dir: &str,
) -> io::Result<HashSet<(ObjectType, String)>> {
    let mut missing: HashSet<(ObjectType, String)> = HashSet::new();

    if haves.contains(want) {
        return Ok(missing)
    }

    if let Ok(commit) = CommitHashes::new(want, git_dir) {
        missing.insert((ObjectType::Commit, commit.hash.to_string()));

        let tree_objects = get_objects_tree_objects(&commit.tree, git_dir)?;
        missing.extend(tree_objects);

        for parent in commit.parent {
            let _missing = get_missing_objects_from2(&parent, haves, git_dir)?;
            missing.extend(_missing);
        }
    }

    Ok(missing)
}

#[derive(Debug, Default)]
struct CommitHashes {
    pub hash: String,
    pub tree: String,
    pub parent: Vec<String>,
}

impl CommitHashes {
    pub fn new(hash: &str, git_dir: &str) -> io::Result<Self> {
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
