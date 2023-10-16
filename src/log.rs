use crate::cat_file;
use std::{
    fs,
    io::{self, Error}, fmt::{Display, format}
};

#[derive(Debug, Default, Clone)]
struct Log {
    git_dir: String,
    commit_hash: String,
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
    author: String,
    committer: String,
    date: String
}

fn invalid_head_error() -> Error {
    Error::new(io::ErrorKind::InvalidData, "HEAD file has invalid data")
}

impl Log {
    pub fn load(git_dir: &str) -> io::Result<Self> {
        let head_path = format!("{}/HEAD", git_dir);
        let head_content = fs::read_to_string(head_path)?;
        let last_commit_ref = head_content.trim().split(": ").last();
        if let Some(commit_ref) = last_commit_ref {
            let commit_ref = commit_ref;
            let heads_path = format!("{}/{}", git_dir, commit_ref);
            match fs::read_to_string(heads_path) {
                Ok(hash) => Self::new_from_hash(&hash.trim(), git_dir),
                Err(_) => Self::new_from_hash(commit_ref, git_dir),
            }
            // if Path::new(&heads_path).exists() {
            //     let hash = fs::read_to_string(heads_path)?;
            //     Self::new_from_hash(&hash, git_dir)
            // } else {
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
                        log.author = fields[0..len-2].join(" ");
                        log.date = fields[len-2..].join(" ")
                    },
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
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let commit = format!("\x1b[0;33mcommit {}\x1b[0m", &self.commit_hash);
        let author = format!("Author: {}", &self.author);
        let date = format!("Date: {}", &self.date);
        let message_vec: Vec<String> = self.message.lines().map(|line| format!("\t{}", line)).collect();
        let message = message_vec.join("\n");
        writeln!(f, "{}\n{}\n{}\n\n{}", commit, author, date, message)
    }
}

impl Iterator for Log {
    type Item = Log;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(hash) = &self.parent_hash {
            if let Ok(log) = Self::new_from_hash(hash, &self.git_dir) {
                let actual = self.clone();
                self.clone_from(&log);
                return Some(actual);
            }
        }
        None
    }
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
    fn test_2() {
        let log = Log::new_from_hash("c6e4695d7f410a8c49787c7c87c5b390b56dc53a", ".git");
        assert!(log.is_ok());
        let log = log.unwrap();
        let logs: Vec<Log>= log.take(5).collect();
        for log in logs {
            println!("{}", log)
        }
    }
}
