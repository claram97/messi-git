use crate::{
    cat_file, configuration::LOGGER_COMMANDS_FILE, logger::Logger, utils::get_current_time,
};
use chrono::{TimeZone, Utc};
use std::{
    fmt::Display,
    fs,
    io::{self, Error, Write},
    path::Path,
};

const DATE_ZERO: &str = "Thu Jan 1 00:00:00 1970 +0000";
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
        match commit {
            Some(hash) => Self::load_from_hash(hash, git_dir),
            None => Self::load_from_head(git_dir),
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
        match line.split_once(' ') {
            Some(("tree", hash)) => {
                self.tree_hash = hash.to_string();
            }
            Some(("parent", hash)) => {
                self.parent_hash = Some(hash.to_string());
            }
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

    /// Set the oneline mode for formatting and return a new instance with the updated configuration.
    ///
    /// This method modifies the current configuration by toggling the oneline mode, which affects
    /// how the formatter outputs information. After setting the oneline mode, it returns a new
    /// instance of the configuration with the updated setting.
    ///
    /// # Arguments
    ///
    /// * `oneline` - A boolean value indicating whether the oneline mode should be enabled (`true`)
    ///               or disabled (`false`).
    ///
    /// # Returns
    ///
    /// Returns a new instance of the configuration with the oneline mode updated according to the
    /// provided boolean value.
    ///
    fn set_oneline(mut self, oneline: bool) -> Self {
        self.oneline = oneline;
        self
    }

    /// Retrieve the parent log of the current commit.
    ///
    /// This method attempts to load the log of the commit's parent, if it exists. If successful,
    /// it returns an `Option<Log>` containing the parent log with an updated oneline mode, otherwise
    /// it returns `None`.
    ///
    /// # Returns
    ///
    /// Returns an `Option<Log>` containing the parent log with an updated oneline mode if the parent
    /// commit exists and can be loaded successfully. Returns `None` otherwise.
    ///
    fn get_parent_log(&self) -> Option<Self> {
        if let Some(parent) = &self.parent_hash {
            if let Ok(log) = Log::load_from_hash(parent, &self.git_dir) {
                return Some(log.set_oneline(self.oneline));
            }
        }
        None
    }

    fn get_formatted_date(&self) -> String {
        let (secs, offset) = &self.date.split_once(' ').unwrap_or(("0", "0"));
        let secs = secs.parse::<i64>().unwrap_or(0);
        let offset_int = offset.parse::<i64>().unwrap_or(0) * 36;
        match Utc.timestamp_opt(secs + offset_int, 0) {
            chrono::LocalResult::Single(date) => {
                let date = date.format("%a %b %e %T %Y");
                format!("{} {}", date, offset)
            }
            _ => DATE_ZERO.to_string(),
        }
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
        let date = self.get_formatted_date();
        let date = format!("Date: {}", date);
        writeln!(f, "{}\n{}\n{}\n\n{}", commit, author, date, message)
    }
}

/// Logs the 'git log' command with the specified commit and Git directory.
///
/// This function logs the 'git log' command with the provided commit and Git directory
/// to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `git_dir` - A `Path` representing the path to the Git directory.
/// * `commit` - An optional string slice representing the complete hash of the commit.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
fn log_log(git_dir: &Path, commit: Option<&str>) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
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
    log_log(Path::new(git_dir), commit)?;
    println!(
        "Calling git log with commit {:?} and git_dir {:?}",
        commit, git_dir
    );
    let log = Log::load(commit, git_dir)?.set_oneline(oneline);
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
    use crate::{add, commit, configuration::GIT_DIR_FOR_TEST, init};

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

    #[test]
    fn test_commits_date() -> io::Result<()> {
        let git_dir_path = "tests/log/";
        init::git_init(git_dir_path, GIT_DIR_FOR_TEST, "master", None)?;
        let git_dir_path = &format!("{}/.mgit", git_dir_path);

        let index = format!("{}/index", git_dir_path);
        let filepath = format!("{}/test_file", git_dir_path);
        fs::File::create(Path::new(&filepath))?;
        add::add(&filepath, &index, git_dir_path, "", None)?;

        let message = "test commit";
        commit::new_commit(git_dir_path, message, "")?;
        // ------------------------------------------------
        let filepath = format!("{}/test_file2", git_dir_path);
        fs::File::create(Path::new(&filepath))?;
        add::add(&filepath, &index, git_dir_path, "", None)?;

        let message = "test commit 2";
        commit::new_commit(git_dir_path, message, "")?;

        let log_iter = log(None, git_dir_path, 2, 0, false)?;

        let logs = accumulate_logs(log_iter);
        println!("{}", logs);

        std::fs::remove_dir_all(git_dir_path)
    }
}
