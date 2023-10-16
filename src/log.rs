use crate::cat_file;
use std::{
    fmt::Display,
    fs,
    io::{self, Error}, path::Path,
};

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

fn invalid_head_error() -> Error {
    Error::new(io::ErrorKind::InvalidData, "HEAD file has invalid data")
}

impl Log {
    pub fn load(commit: Option<&str>, git_dir: &str) -> io::Result<Self> {
        if let Some(hash) = commit {
            Self::new_from_hash(hash, git_dir)
        } else {
            Self::load_from_head(git_dir)
        }
    }

    fn load_from_head(git_dir: &str) -> io::Result<Self> {
        let head_path = format!("{}/HEAD", git_dir);
        let head_content = fs::read_to_string(head_path)?;
        let last_commit_ref = head_content.trim().split(": ").last();
        if let Some(commit_ref) = last_commit_ref {
            let heads_path = format!("{}/{}", git_dir, commit_ref);
            dbg!(&heads_path);
            match fs::read_to_string(heads_path) {
                Ok(hash) => Self::new_from_hash(&hash.trim(), git_dir),
                Err(_) => Self::new_from_hash(commit_ref, git_dir),
            }
            // if Path::new(&heads_path).exists() {
            //     let hash = fs::read_to_string(heads_path)?;
            //     Self::new_from_hash(&hash, git_dir)
            // } else {
            //     dbg!(&commit_ref);
            //     Self::new_from_hash(commit_ref, git_dir)
            // }
        } else {
            Err(invalid_head_error())
        }
    }

    fn new_from_hash(hash: &str, git_dir: &str) -> io::Result<Self> {
        let commit_content = cat_file::cat_file_return_content(hash, git_dir)?;
        let mut log = Self::default();
        log.git_dir = git_dir.to_string();
        log.commit_hash = hash.to_string();

        let header_lines = commit_content.lines().position(|line| line.is_empty());

        if let Some(n) = header_lines {
            for line in commit_content.lines().take(n) {
                match line.split_once(' ') {
                    Some(("tree", hash)) => log.tree_hash = hash.to_string(),
                    Some(("parent", hash)) => log.parent_hash = Some(hash.to_string()),
                    Some(("author", author)) => {
                        let fields: Vec<&str> = author.split(' ').collect();
                        let len = fields.len();
                        log.author = fields[0..len - 2].join(" ");
                        log.date = fields[len - 2..].join(" ")
                    }
                    Some(("committer", committer)) => log.committer = committer.to_string(),
                    _ => return Err(invalid_head_error()),
                }
            }
            log.message = commit_content.lines().skip(n).collect();
            Ok(log)
        } else {
            Err(invalid_head_error())
        }
    }

    fn get_parent_log(&self) -> Option<Self> {
        if let Some(parent) = &self.parent_hash {
            let next_log = Log::new_from_hash(&parent, &self.git_dir);
            match next_log {
                Ok(mut log) => {
                    log.oneline = self.oneline;
                    Some(log)
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> LogIter {
        LogIter::new(self.clone())
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let commit = format!("\x1b[0;33mcommit {}\x1b[0m", &self.commit_hash);
        let message_vec: Vec<String> = self
            .message
            .lines()
            .map(|line| format!("\t{}", line))
            .collect();
        let message = message_vec.join("\n");

        if self.oneline {
            let commit = commit.replace("commit ", "");
            return writeln!(f, "{} {}", commit, message);
        }

        let author = format!("Author: {}", &self.author);
        let date = format!("Date: {}", &self.date);
        writeln!(f, "{}\n{}\n{}\n\n{}", commit, author, date, message)
    }
}

pub fn log(
    commit: Option<&str>,
    git_dir: &str,
    amount: usize,
    skip: usize,
    oneline: bool,
) -> io::Result<impl Iterator<Item = Log>> {
    let mut log = Log::load(commit, git_dir)?;
    log.oneline = oneline;

    Ok(log.iter().skip(skip).take(amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let log = Log::new_from_hash("c6e4695d7f410a8c49787c7c87c5b390b56dc53a", ".git");
        assert!(log.is_ok())
    }

    #[test]
    fn test_3() {
        let log = log(
            Some("c6e4695d7f410a8c49787c7c87c5b390b56dc53a"),
            ".git",
            5,
            0,
            true,
        );
        assert!(log.is_ok());
        let log = log.unwrap();
        for l in log {
            println!("{}", l)
        }
    }

    #[test]
    fn test_4() {
        let log = log(
            None,
            ".mgit",
            5,
            0,
            true,
        );
        assert!(log.is_ok());
        let log = log.unwrap();
        for l in log {
            println!("{}", l);
        }
    }
}
