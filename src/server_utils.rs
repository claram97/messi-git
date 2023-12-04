use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Error, Read, Write},
    net::TcpStream,
    path::PathBuf,
    str::from_utf8,
};

use crate::{cat_file, logger, utils::get_current_time};

pub fn log(message: &str) -> io::Result<()> {
    let mut logger = logger::Logger::new("logs/log.log")?;
    let message = message.replace('\0', "\\0").replace('\n', "\\n");
    let message = format!("{} - {}", get_current_time(), message);
    write!(logger, "{}", message)?;
    logger.flush()
}

// HELPER MODULE

/// Creates an error indicating that the connection was not established.
///
/// This function returns an `io::Error` with the error kind set to `BrokenPipe` and a message
/// indicating that the operation failed because the connection was not established.
///
/// # Returns
///
/// An `io::Error` indicating that the connection was not established.
///
pub fn connection_not_established_error() -> Error {
    Error::new(
        io::ErrorKind::BrokenPipe,
        "The operation failed because the connection was not established.",
    )
}

/// Read a line in PKT format in a TcpStream
/// Returns the size of the line and its content as string
pub fn read_pkt_line(socket: &mut TcpStream) -> io::Result<(usize, String)> {
    let (size, bytes) = read_pkt_line_bytes(socket)?;
    let line = from_utf8(&bytes).unwrap_or_default().to_string();
    if line.starts_with("ERR") {
        return Err(Error::new(io::ErrorKind::Other, format!("Error: {}", line)));
    }
    log(&format!(
        "Reading line of size {} -> {}",
        size,
        line.replace('\0', "\\0")
    ))?;
    Ok((size, line))
}

/// Read a line in PKT format in a TcpStream
/// Returns the size of the line and its content as bytes
pub fn read_pkt_line_bytes(socket: &mut TcpStream) -> io::Result<(usize, Vec<u8>)> {
    let mut buf = vec![0u8; 4];
    socket.read_exact(&mut buf)?;

    let size = from_utf8(&buf).unwrap_or_default();
    let size = usize::from_str_radix(size, 16).unwrap_or(0);

    if size <= 4 {
        return Ok((size, vec![]));
    }

    let mut buf = vec![0u8; size - 4];
    socket.read_exact(&mut buf)?;
    log(&format!("Reading bytes of size {} -> {:?}", size, buf))?;
    Ok((size, buf))
}

/// Given a text to send a git client/server, this function transform it to a
/// string in PKT format
pub fn pkt_line(line: &str) -> String {
    let len = line.len() + 4; // len
    let mut len_hex = format!("{len:x}");
    while len_hex.len() < 4 {
        len_hex = "0".to_owned() + &len_hex
    }
    len_hex + line
}

/// Given some bytes to send a git client/server, this function transform it
/// in PKT format
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

/// Gets the ref name of a branch
/// If branch is HEAD, then it gets the ref name of the branch pointed by HEAD
pub fn get_head_from_branch(git_dir: &str, branch: &str) -> io::Result<String> {
    if branch == "HEAD" {
        let head = PathBuf::from(git_dir).join("HEAD");
        let content = fs::read_to_string(head)?;
        let (_, head) = content.rsplit_once(": ").ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid data HEAD. Must have ref for fetch: {}", content),
        ))?;
        return Ok(head.trim().to_string());
    }
    let pathbuf = PathBuf::from(git_dir)
        .join("refs")
        .join("tags")
        .join(branch);
    if pathbuf.exists() {
        Ok(format!("refs/tags/{}", branch))
    } else {
        Ok(format!("refs/heads/{}", branch))
    }
}

/// Auxiliar function which get refs under refs/heads
pub fn get_head_tags_refs(git_dir: &str) -> io::Result<HashMap<String, String>> {
    let pathbuf = PathBuf::from(git_dir);
    let heads = pathbuf.join("refs").join("heads");
    let tags = pathbuf.join("refs").join("tags");
    let mut refs = get_refs(heads)?;
    let tags = get_refs(tags)?;
    refs.extend(tags);
    Ok(refs)
}

/// Auxiliar function which get refs under refs/heads
pub fn get_client_refs(git_dir: &str, remote: &str) -> io::Result<HashMap<String, String>> {
    let pathbuf = PathBuf::from(git_dir);
    let remotes = pathbuf.join("refs").join("remotes").join(remote);
    if !remotes.exists() {
        fs::create_dir_all(&remotes)?;
    }
    let tags = pathbuf.join("refs").join("tags");
    let mut refs = get_refs(remotes)?;
    let tags = get_refs(tags)?;
    refs.extend(tags);
    Ok(refs)
}

// Auxiliar function which get refs under refs_path
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

/// Parse a line in PKT format with the format: want|have hash
pub fn parse_line_want_have(line: &str) -> io::Result<(WantHave, String)> {
    let (want_or_have, hash) = line.split_once(' ').ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Invalid want line: {}", line),
    ))?;

    let t = WantHave::try_from(want_or_have)?;
    let (hash, _) = hash.split_once(' ').unwrap_or((hash, ""));
    Ok((t, hash.trim().to_string()))
}

/// Get missing objects of a repository
/// It returns a set of tuples with the object type and the hash
///
/// Parameters:
///     - want: hash of the object to get
///     - haves: set of hashes of the objects that the caller has
///     - git_dir: path to the git directory
pub fn get_missing_objects_from(
    want: &str,
    haves: &HashSet<String>,
    git_dir: &str,
) -> io::Result<Vec<String>> {
    if haves.contains(want) {
        return Ok(vec![]);
    }

    let mut missing: HashSet<String> = HashSet::new();
    if let Ok(commit) = CommitHashes::new(want, git_dir) {
        missing.insert(commit.hash.to_string());

        let tree_objects = get_objects_tree_objects(&commit.tree, git_dir)?;
        missing.extend(tree_objects);

        for parent in commit.parent {
            let _missing = get_missing_objects_from(&parent, haves, git_dir)?;
            missing.extend(_missing);
        }
    }
    let mut v: Vec<String> = missing.into_iter().collect();
    v.sort();
    Ok(v)
}

#[derive(Debug, Default)]
struct CommitHashes {
    pub hash: String,
    pub tree: String,
    pub parent: Vec<String>,
}

impl CommitHashes {
    /// Creates a new commit instance by loading commit information from a given hash.
    ///
    /// This function reads the commit content associated with the provided hash in the Git repository
    /// specified by `git_dir`. It then parses the commit information and returns a `Commit` instance.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the commit to load.
    /// * `git_dir` - The path to the Git repository.
    ///
    /// # Returns
    ///
    /// A Result containing the new `Commit` instance if successful, or an `io::Error` if an error occurs.
    ///
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

    /// Parses a single line from a commit header and updates the commit instance accordingly.
    ///
    /// This function is part of the commit loading process and is responsible for interpreting
    /// specific lines in the commit header, such as the tree reference and parent references.
    ///
    /// # Arguments
    ///
    /// * `line` - A single line from the commit header to be parsed.
    ///
    fn parse_commit(&mut self, line: &str) {
        match line.split_once(' ') {
            Some(("tree", hash)) => self.tree = hash.to_string(),
            Some(("parent", hash)) => self.parent.push(hash.to_string()),
            _ => {}
        }
    }
}

/// Recursively retrieves objects (trees and blobs) associated with a given tree hash.
///
/// This function traverses the tree structure recursively and collects the object references
/// (hashes) along with their corresponding types (tree or blob). The result is a HashSet
/// containing tuples of object types and their respective hashes.
///
/// # Arguments
///
/// * `hash` - The hash of the tree for which objects are to be retrieved.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// An `io::Result` containing a `HashSet` of tuples representing object types and their hashes.
///
fn get_objects_tree_objects(hash: &str, git_dir: &str) -> io::Result<HashSet<String>> {
    let mut objects: HashSet<String> = HashSet::new();
    objects.insert(hash.to_string());
    let content = cat_file::cat_tree(hash, git_dir)?;

    for (mode, _, hash) in content {
        if mode == "40000 " || mode == "40000" {
            let tree_objects = get_objects_tree_objects(&hash, git_dir)?;
            objects.extend(tree_objects);
        } else {
            objects.insert(hash.to_string());
        };
    }

    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkt_line() {
        assert_eq!(pkt_line("hello"), "0009hello");
        assert_eq!(
            pkt_line("git-upload-pack /repo\0host=localhost\0\0version=1\0"),
            "0034git-upload-pack /repo\0host=localhost\0\0version=1\0"
        );
        assert_eq!(pkt_line("want 86135720c1283d83f2744781a915aba3d74da37b multi_ack include-tag side-band-64k ofs-delta\n"), "0060want 86135720c1283d83f2744781a915aba3d74da37b multi_ack include-tag side-band-64k ofs-delta\n");
    }

    #[test]
    fn test_get_head_from_branch() -> io::Result<()> {
        let head = get_head_from_branch("tests/packfiles/.mgit", "master")?;
        assert_eq!(head, "refs/heads/master");
        let head = get_head_from_branch("tests/packfiles/.mgit", "v1.0")?;
        assert_eq!(head, "refs/tags/v1.0");
        let head = get_head_from_branch("tests/packfiles/.mgit", "HEAD")?;
        assert_eq!(head, "refs/heads/master");
        Ok(())
    }

    #[test]
    fn test_get_refs() -> io::Result<()> {
        let refs = get_refs(
            PathBuf::from("tests/packfiles/.mgit")
                .join("refs")
                .join("heads"),
        )?;
        assert!(refs.contains_key(&"master".to_string()));
        let refs = get_refs(
            PathBuf::from("tests/packfiles/.mgit")
                .join("refs")
                .join("tags"),
        )?;
        assert!(refs.contains_key(&"v1.0".to_string()));
        let refs = get_refs(
            PathBuf::from("tests/packfiles/.mgit")
                .join("refs")
                .join("remotes")
                .join("origin"),
        )?;
        assert!(refs.contains_key(&"master".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_head_tags_refs() -> io::Result<()> {
        let refs = get_head_tags_refs("tests/packfiles/.mgit")?;
        assert!(refs.contains_key(&"master".to_string()));
        assert!(refs.contains_key(&"v1.0".to_string()));
        Ok(())
    }

    #[test]
    fn test_client_refs() -> io::Result<()> {
        let refs = get_client_refs("tests/packfiles/.mgit", "origin")?;
        assert!(refs.contains_key(&"master".to_string()));
        assert!(refs.contains_key(&"v1.0".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_line_want_have() -> io::Result<()> {
        let (want_have, hash) = parse_line_want_have("want 86135720c1283d83f2744781a915aba3d74da37b multi_ack include-tag side-band-64k ofs-delta\n")?;
        assert_eq!(want_have, WantHave::Want);
        assert_eq!(hash, "86135720c1283d83f2744781a915aba3d74da37b");
        let (want_have, hash) = parse_line_want_have("have 86135720c1283d83f2744781a915aba3d74da37b multi_ack include-tag side-band-64k ofs-delta\n")?;
        assert_eq!(want_have, WantHave::Have);
        assert_eq!(hash, "86135720c1283d83f2744781a915aba3d74da37b");
        Ok(())
    }
}
