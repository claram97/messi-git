use crate::cat_file;
use std::{
    fmt::Display,
    fs,
    io::{self, Error},
    path::Path,
};
use crate::logger::Logger;
use std::io::Write;
use crate::utils::get_current_time;

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

/// Creates a custom `io::Error` with the `InvalidData` kind, representing an error due to
/// invalid data encountered during processing.
///
/// This function takes a string `commit` as a parameter and constructs an `io::Error` with
/// `InvalidData` kind, providing additional information about the commit causing the error.
///
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
            println!("Loading form hash {:?}", hash);
            Self::load_from_hash(hash, git_dir)
        } else {
            println!("Loading from head {:?}", git_dir);
            Self::load_from_head(git_dir)
        }
    }

    /// Load the current commit from the HEAD reference in the specified Git directory.
    ///
    /// This function reads the contents of the HEAD file, extracts the reference to the last commit,
    /// and loads the corresponding commit from either the heads directory or directly using the
    /// commit hash. The commit is then returned as a result.
    ///
    /// # Arguments
    ///
    /// * `git_dir` - A string representing the path to the Git directory.
    ///
    /// # Returns
    ///
    /// Returns a result containing the loaded commit on success, or an `io::Error` on failure.
    ///
    fn load_from_head(git_dir: &str) -> io::Result<Self> {
        println!("loading from head");
        let head_path = format!("{}/HEAD", git_dir);
        println!("head path is {:?}\n", head_path);
        let head_content = fs::read_to_string(head_path)?;
        println!("head content is {}\n", head_content);
        let last_commit_ref = head_content.trim().split(": ").last();
        println!("last commit ref is {:?}\n", last_commit_ref);
        match last_commit_ref {
            Some(refs) => {
                let heads_path = format!("{}/{}", git_dir, refs);
                println!("heads path is {:?}\n", heads_path);
                if Path::new(&heads_path).exists() {
                    let hash = fs::read_to_string(heads_path)?;
                    println!("hash is {:?}\n", hash);
                    Self::load_from_hash(hash.trim(), git_dir)
                } else {
                    println!("path doesn't exist, hash is {:?}\n", refs);
                    Self::load_from_hash(refs, git_dir)
                }
            }
            None => Err(invalid_data_error(&head_content)),
        }
    }

    /// Load a commit from a given commit hash in the specified Git directory.
    ///
    /// This function retrieves the content of the commit using `cat-file`, parses the commit header
    /// lines to extract relevant information, and constructs a `Commit` struct. The commit's message,
    /// Git directory path, and commit hash are set based on the parsed content. The resulting commit
    /// is returned as a result.
    ///
    /// # Arguments
    ///
    /// * `hash` - A string representing the commit hash.
    /// * `git_dir` - A string representing the path to the Git directory.
    ///
    /// # Returns
    ///
    /// Returns a result containing the loaded commit on success, or an `io::Error` on failure.
    ///
    fn load_from_hash(hash: &str, git_dir: &str) -> io::Result<Self> {
        println!("Loading from hash\n");
        println!("Hash is {:?}\n", hash);
        let commit_content = cat_file::cat_file_return_content(hash, git_dir)?;
        println!("commit content is {:?}\n", commit_content);
        let header_lines = commit_content.lines().position(|line| line.is_empty());
        println!("header lines {:?}\n", header_lines);
        match header_lines {
            Some(n) => {
                let mut log = Self::default();
                println!("log default {:?}\n", log);
                for line in commit_content.lines().take(n) {
                    println!("line {:?}\n", line);
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

    /// Parse a commit header line and update the relevant fields of the `Commit` struct.
    ///
    /// This function takes a commit header line as input and extracts information such as the tree
    /// hash, parent hash (if present), author details, and committer information. The extracted
    /// information is then used to update the corresponding fields of the `Commit` struct.
    ///
    /// # Arguments
    ///
    /// * `line` - A string representing a single line from the commit header.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the parsing is successful and the fields are updated, or an `io::Error`
    /// if the input line does not match expected patterns.
    ///
    /// # Errors
    ///
    /// This function returns an `io::Error` if the provided line does not conform to the expected
    /// format for commit header lines, or if there is insufficient data to update the commit fields.
    ///
    fn parse_commit_header_line(&mut self, line: &str) -> io::Result<()> {
        println!("Calling parse commit header line function!\n");
        match line.split_once(' ') {
            Some(("tree", hash)) => {self.tree_hash = hash.to_string();
                println!("hash {:?}\n", hash);
            },
            Some(("parent", hash)) => {self.parent_hash = Some(hash.to_string());
                println!("hash {:?}\n", hash);},
            Some(("author", author)) => {
                let fields: Vec<&str> = author.split(' ').collect();
                println!("fields {:?}\n", fields);
                let len = fields.len();
                println!("len {:?}\n", len);
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

    /// Set the online mode for formatting and return a new instance with the updated configuration.
    ///
    /// This method modifies the current configuration by toggling the online mode, which affects
    /// how the formatter outputs information. After setting the online mode, it returns a new
    /// instance of the configuration with the updated setting.
    ///
    /// # Arguments
    ///
    /// * `oneline` - A boolean value indicating whether the online mode should be enabled (`true`)
    ///               or disabled (`false`).
    ///
    /// # Returns
    ///
    /// Returns a new instance of the configuration with the online mode updated according to the
    /// provided boolean value.
    ///
    fn set_online(mut self, oneline: bool) -> Self {
        self.oneline = oneline;
        self
    }

    /// Retrieve the parent log of the current commit.
    ///
    /// This method attempts to load the log of the commit's parent, if it exists. If successful,
    /// it returns an `Option<Log>` containing the parent log with an updated online mode, otherwise
    /// it returns `None`.
    ///
    /// # Returns
    ///
    /// Returns an `Option<Log>` containing the parent log with an updated online mode if the parent
    /// commit exists and can be loaded successfully. Returns `None` otherwise.
    ///
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
            .fold(String::new(), |acc, line| format!("{}\t{}", acc, line));

        if self.oneline {
            let commit = commit.replace("commit ", "");
            return write!(f, "{} {}", commit, message);
        }

        let author = format!("Author: {}", &self.author);
        let date = format!("Date: {}", &self.date);
        writeln!(f, "{}\n{}\n{}\n\n{}", commit, author, date, message)
    }
}
fn log_log(git_dir: &Path, commit: Option<&str>) -> io::Result<()> {
    let log_file_path = "logger_comands.txt";
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git log': Commit '{:?}', Git Dir '{}', {}",
        commit,
        git_dir.display(),
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
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
    log_log(&Path::new(git_dir), commit)?;
    println!("Calling git log with commit {:?} and git_dir {:?}", commit, git_dir);
    let log = Log::load(commit, git_dir)?.set_online(oneline);
    Ok(log.iter().skip(skip).take(amount))
}

/// Print logs from an iterator.
///
/// This function takes an iterator of logs and prints each log to the console. It is a convenient
/// way to display commit information directly from an iterator, such as the result of iterating
/// over a commit history.
///
/// # Arguments
///
/// * `log_iter`: An iterator yielding instances of `Log` representing commit information.
///
pub fn print_logs(log_iter: impl Iterator<Item = Log>) {
    for log in log_iter {
        println!("{log}")
    }
}

/// Accumulate logs from an iterator into a single string.
///
/// This function takes an iterator of logs and concatenates their string representations into
/// a single string. It can be useful when you want to accumulate commit information for further
/// processing or display.
///
/// # Arguments
///
/// * `log_iter`: An iterator yielding instances of `Log` representing commit information.
///
/// # Returns
///
/// A `String` containing the concatenated string representations of the logs.
///
pub fn accumulate_logs(log_iter: impl Iterator<Item = Log>) -> String {
    let mut log_text = String::new();

    for log in log_iter {
        log_text.push_str(&format!("{log}\n"));
    }

    log_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_oneline() {
        let log_iter = log(
            Some("2d2d2887951eaf42f37b437d44bb4cfcae97fe54"),
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
    #[ignore]
    fn test_many_lines() {
        let log_iter = log(
            Some("2d2d2887951eaf42f37b437d44bb4cfcae97fe54"),
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
    #[ignore]
    fn test_from_head() {
        let log_iter = log(None, ".git", 3, 0, true);
        assert!(log_iter.is_ok());
        let log_iter = log_iter.unwrap();
        print_logs(log_iter)
    }
}
