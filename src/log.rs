use crate::cat_file;
use std::{
    fmt::Display,
    fs,
    io::{self, Error},
    path::Path,
};

/// LogIter is a structure that will help to iterate
/// through commit logs in the correct way.
///
/// Also implements Iterator trait so it has a lot
/// of flexibility because of that
pub struct LogIter {
    log: Option<Log>,
}

impl LogIter {
    fn new(log: Log) -> Self {
        Self { log: Some(log) }
    }
}

impl Iterator for LogIter {
    type Item = Log;

    fn next(&mut self) -> Option<Self::Item> {
        let actual = self.log.clone();
        if let Some(log) = &self.log {
            self.log = log.get_parent_log();
        }
        actual
    }
}

/// Log is a structure that will manage all relevant information
/// about each commit.
///
/// A log can be loaded in two different ways:
/// - Giving a commit hash
///
/// - Not giving a commit hash. In this case, HEAD file of
/// the repo will be read.
///
/// The method 'iter()' is available to get a LogIter instance
/// starting in this Log.
///
/// The load of the Log may fail because of I/O errors.
///
/// For example: if the user try to load a Log from an inexistent commit hash,
/// will fail.
#[derive(Debug, Default, Clone)]
pub struct Log {
    git_dir: String,
    commit_hash: String,
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
    author: String,
    date: String,
    committer: String,
    oneline: bool,
}

fn invalid_data_error(commit: &str) -> Error {
    Error::new(io::ErrorKind::InvalidData, format!("Commit: {}", commit))
}

impl Log {
    /// Method to load and return a Log
    ///
    /// The git directory is needed for some internal actions.
    ///
    /// The commit may or may not be present.
    ///
    /// If available, the log of the given commit is loaded.
    ///
    /// Otherwise, HEAD file will be read to load the Log.
    ///
    /// The load of the Log may fail because of I/O errors.
    pub fn load(commit: Option<&str>, git_dir: &str) -> io::Result<Self> {
        if let Some(hash) = commit {
            Self::load_from_hash(hash, git_dir)
        } else {
            Self::load_from_head(git_dir)
        }
    }

    fn load_from_head(git_dir: &str) -> io::Result<Self> {
        let head_path = format!("{}/HEAD", git_dir);
        let head_content = fs::read_to_string(head_path)?;
        let last_commit_ref = head_content.trim().split(": ").last();

        match last_commit_ref {
            Some(refs) => {
                let heads_path = format!("{}/{}", git_dir, refs);
                if Path::new(&heads_path).exists() {
                    let hash = fs::read_to_string(heads_path)?;
                    Self::load_from_hash(hash.trim(), git_dir)
                } else {
                    Self::load_from_hash(refs, git_dir)
                }
            }
            None => Err(invalid_data_error(&head_content)),
        }
    }

    fn load_from_hash(hash: &str, git_dir: &str) -> io::Result<Self> {
        let commit_content = cat_file::cat_file_return_content(hash, git_dir)?;
        let header_lines = commit_content.lines().position(|line| line.is_empty());

        match header_lines {
            Some(n) => {
                let mut log = Self::default();
                for line in commit_content.lines().take(n) {
                    log.parse_commit_header_line(line)?;
                }
                log.message = commit_content.lines().skip(n).collect();
                log.git_dir = git_dir.to_string();
                log.commit_hash = hash.to_string();
                Ok(log)
            }
            None => Err(invalid_data_error(hash)),
        }
    }

    fn parse_commit_header_line(&mut self, line: &str) -> io::Result<()> {
        match line.split_once(' ') {
            Some(("tree", hash)) => self.tree_hash = hash.to_string(),
            Some(("parent", hash)) => self.parent_hash = Some(hash.to_string()),
            Some(("author", author)) => {
                let fields: Vec<&str> = author.split(' ').collect();
                let len = fields.len();
                if len < 4 {
                    return Err(invalid_data_error(line));
                }
                self.author = fields[0..len - 2].join(" ");
                self.date = fields[len - 2..].join(" ")
            }
            Some(("committer", committer)) => self.committer = committer.to_string(),
            _ => {}
        }
        Ok(())
    }

    fn set_online(mut self, oneline: bool) -> Self {
        self.oneline = oneline;
        self
    }

    fn get_parent_log(&self) -> Option<Self> {
        if let Some(parent) = &self.parent_hash {
            if let Ok(log) = Log::load_from_hash(parent, &self.git_dir) {
                return Some(log.set_online(self.oneline));
            }
        }
        None
    }

    /// Returns an iterator starting in 'self'
    ///
    /// When accessing to the next Log, it refers to 'parent log'
    ///
    /// self is consumed
    pub fn iter(self) -> LogIter {
        LogIter::new(self)
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let commit = format!("\x1b[0;33mcommit {}\x1b[0m", &self.commit_hash);
        let message = self
            .message
            .lines()
            .map(|line| format!("\t{}", line))
            .collect::<String>();

        if self.oneline {
            let commit = commit.replace("commit ", "");
            return write!(f, "{} {}", commit, message);
        }

        let author = format!("Author: {}", &self.author);
        let date = format!("Date: {}", &self.date);
        writeln!(f, "{}\n{}\n{}\n\n{}", commit, author, date, message)
    }
}

/// This function receive relevante information to create a Log and
/// return the corresponding iterator
///
/// The user who calls this function will have an iterator of logs
/// to use. Usually it will be used for printing in stdout
pub fn log(
    commit: Option<&str>,
    git_dir: &str,
    amount: usize,
    skip: usize,
    oneline: bool,
) -> io::Result<impl Iterator<Item = Log>> {
    let log = Log::load(commit, git_dir)?.set_online(oneline);
    Ok(log.iter().skip(skip).take(amount))
}

pub fn print_logs(log_iter: impl Iterator<Item = Log>) {
    for log in log_iter {
        println!("{log}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oneline() {
        let log_iter = log(
            Some("c6e4695d7f410a8c49787c7c87c5b390b56dc53a"),
            ".git",
            5,
            0,
            true,
        );
        assert!(log_iter.is_ok());
        let log_iter = log_iter.unwrap();
        print_logs(log_iter)
    }

    #[test]
    fn test_many_lines() {
        let log_iter = log(
            Some("c6e4695d7f410a8c49787c7c87c5b390b56dc53a"),
            ".git",
            5,
            0,
            false,
        );
        assert!(log_iter.is_ok());
        let log_iter = log_iter.unwrap();
        print_logs(log_iter)
    }

    #[test]
    fn test_from_head() {
        let log_iter = log(None, ".git", 3, 0, true);
        assert!(log_iter.is_ok());
        let log_iter = log_iter.unwrap();
        print_logs(log_iter)
    }
}
