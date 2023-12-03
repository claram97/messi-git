use crate::configuration::{GIT_DIR, HOST, LOGGER_COMMANDS_FILE, REMOTE};
use crate::logger::Logger;
use crate::utils::get_current_time;
use crate::{client::Client, config};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
};

/// Represents a single entry in the "FETCH_HEAD" file, typically created during Git fetch operations.
///
/// Each `FetchEntry` contains information about a fetched commit, including its commit hash, the branch
/// name from which it was fetched, and the URL of the remote repository from which it was retrieved.
///
/// # Fields
///
/// * `commit_hash`: The commit hash of the fetched commit.
/// * `branch_name`: The name of the branch from which the commit was fetched.
/// * `remote_repo_url`: The URL of the remote repository from which the commit was retrieved.
///
pub struct FetchEntry {
    pub commit_hash: String,
    pub branch_name: String,
    remote_repo_url: String,
}

/// Represents the "FETCH_HEAD" file generated during Git fetch operations, containing a list of fetched entries.
///
/// The `FetchHead` struct maintains a list of `FetchEntry` instances, each representing a fetched commit and
/// associated details. This structure is often used to keep track of the commits fetched from a remote repository.
///
/// # Fields
///
/// * `entries`: A vector of `FetchEntry` instances representing fetched commits.
///
pub struct FetchHead {
    entries: Vec<FetchEntry>,
}

impl Default for FetchHead {
    fn default() -> Self {
        Self::new()
    }
}

impl FetchHead {
    /// Creates a new, empty FetchHead instance.
    ///
    /// This method returns a new, empty `FetchHead` struct with no fetched entries.
    ///
    pub fn new() -> FetchHead {
        FetchHead {
            entries: Vec::new(),
        }
    }

    /// Adds a FetchEntry to the FetchHead instance.
    ///
    /// This method adds a `FetchEntry` to the `FetchHead` by pushing it to the list of fetched entries.
    ///
    /// # Arguments
    ///
    /// * `entry`: The `FetchEntry` to be added to the `FetchHead`.
    ///
    pub fn add_entry(&mut self, entry: FetchEntry) {
        self.entries.push(entry);
    }

    /// Retrieves a reference to the list of fetched entries.
    ///
    /// This method returns a reference to the list of `FetchEntry` instances contained in the `FetchHead`.
    ///
    pub fn get_entries(&self) -> &Vec<FetchEntry> {
        &self.entries
    }

    /// Retrieves a reference to a specific FetchEntry by branch name.
    ///
    /// This method searches for a `FetchEntry` with a matching branch name and returns a reference to it.
    /// If no matching entry is found, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `branch_name`: The branch name to search for.
    ///
    pub fn get_branch_entry(&self, branch_name: &str) -> Option<&FetchEntry> {
        self.entries
            .iter()
            .find(|&entry| entry.branch_name == branch_name)
    }

    /// Writes the contents of the FetchHead to a file.
    ///
    /// This method writes the entries of the `FetchHead` to a file specified by the `path`.
    ///
    /// # Arguments
    ///
    /// * `path`: The path to the file where the contents will be written.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
    ///
    pub fn write_file(&self, path: &str) -> io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for entry in &self.entries {
            let line = format!(
                "{} {} of {}\n",
                entry.commit_hash, entry.branch_name, entry.remote_repo_url
            );
            file.write_all(line.as_bytes())?;
        }
        Ok(())
    }

    /// Loads the contents of a FetchHead from a file.
    ///
    /// This method reads and loads the contents of a `FetchHead` from a file specified by the `path`.
    ///
    /// # Arguments
    ///
    /// * `path`: The path to the file from which the contents will be loaded.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. In case of success, an `io::Result<FetchHead>` is returned.
    ///
    pub fn load_file(path: &str) -> io::Result<FetchHead> {
        let file = std::fs::File::open(path)?;
        let mut fetch_head = FetchHead::new();

        for line in io::BufReader::new(file).lines() {
            let line = line?;
            let line_split: Vec<&str> = line.split(' ').collect();
            let commit_hash = line_split[0].to_string();
            let branch_name = line_split[1].to_string();
            let remote_repo_url = line_split[3].to_string();
            let entry = FetchEntry {
                commit_hash,
                branch_name,
                remote_repo_url,
            };
            fetch_head.add_entry(entry);
        }

        Ok(fetch_head)
    }
}

/// Cleans a list of Git references by extracting their last components.
///
/// This function takes a vector of Git reference strings and extracts the last component of each reference
/// by splitting the string at '/' and taking the last part. It returns a new vector containing only the last
/// components of the references, effectively removing any preceding namespace or hierarchy.
///
/// # Arguments
///
/// * `refs`: A vector of Git reference strings to process.
///
/// # Returns
///
/// Returns a new vector containing only the last components of the input Git references.
///
fn get_clean_refs(refs: &HashMap<String, String>) -> Vec<String> {
    let clean_refs = refs
        .iter()
        .map(|x| match x.0.split('/').last() {
            Some(string) => string.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();
    clean_refs
}

/// Logs the 'git fetch' command with the specified remote repository name, host, and local directory.
///
/// This function logs the 'git fetch' command with the provided remote repository name, host, and
/// local directory to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `remote_repo_name` - An optional name for the remote repository to fetch from. If not provided, "origin" is used.
/// * `host` - The host associated with the remote repository.
/// * `local_dir` - The path to the local directory where the repository is located.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_fetch(remote_repo_name: Option<&str>, host: &str, local_dir: &str) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let repo_name = remote_repo_name.unwrap_or(REMOTE);

    let full_message = format!(
        "Command 'git fetch': Remote Repo Name '{}', Host '{}', Local Dir '{}', {}",
        repo_name,
        host,
        local_dir,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Perform a Git fetch operation to update the local repository with remote changes.
///
/// This function carries out a Git fetch operation, which retrieves the most recent commit of each branch
/// from the remote repository specified by `remote_repo_name`. It also brings any objects that are not
/// present in the local repository. The remote repository is identified by its name, and the host is provided.
/// The fetched commit information is stored in the FETCH_HEAD file.
///
/// # Arguments
///
/// * `remote_repo_name`: An optional name for the remote repository to fetch from. If not provided, "origin" is used.
/// * `host`: The host associated with the remote repository.
/// * `local_dir`: The path to the local directory where the repository is located.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
///
pub fn git_fetch(
    _remote_repo_name: Option<&str>,
    _host: &str,
    local_dir: &str,
) -> io::Result<Vec<String>> {
    let git_dir = local_dir.to_string() + "/" + GIT_DIR;
    let config_file = config::Config::load(&git_dir)?;
    let remote_name = REMOTE;
    let remote_url = config_file.get_url(remote_name, &mut io::stdout())?;
    let (address, repo_name) = match remote_url.rsplit_once('/') {
        Some((address, repo_name)) => (address, repo_name),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid data in remote dir: {}", remote_url),
            ))
        }
    };
    let mut client = Client::new(address, repo_name, HOST);
    let refs = client.get_server_refs()?;
    let clean_refs = get_clean_refs(&refs);
    client.upload_pack(clean_refs.clone(), &git_dir, remote_name)?;
    let fetch_head_path = git_dir.to_string() + "/FETCH_HEAD";
    let mut fetch_head_file = FetchHead::new();

    for server_ref in clean_refs.clone() {
        if server_ref != "HEAD" {
            let server_ref_head = "refs/heads/".to_string() + &server_ref;
            let hash = match refs.get(&server_ref_head) {
                Some(hash) => hash,
                None => {
                    println!("Error: Could not find hash for {}", server_ref);
                    continue;
                }
            };
            let entry = FetchEntry {
                commit_hash: hash.to_string(),
                branch_name: server_ref,
                remote_repo_url: remote_url.clone(),
            };
            fetch_head_file.add_entry(entry);
        }
    }
    fetch_head_file.write_file(&fetch_head_path)?;
    log_fetch(_remote_repo_name, _host, local_dir)?;
    Ok(clean_refs)
}

#[cfg(test)]
mod tests {
    use crate::clone;
    use std::env;
    const PORT: &str = "9418";
    #[ignore = "This test only works if the server is running"]
    #[test]
    fn test_fetch() {
        let local_dir = env::temp_dir().to_str().unwrap().to_string() + "/test_fetch";
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "repo_prueba";
        let host = "localhost";
        let _ = clone::git_clone(&address, remote_repo_name, host, &local_dir);

        let result = super::git_fetch(Some(remote_repo_name), "localhost", &local_dir);
        assert!(result.is_ok());
    }
}
