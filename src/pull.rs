use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::configuration::{GIT_DIR, LOGGER_COMMANDS_FILE};
use crate::logger::Logger;
use crate::utils::get_current_time;
use crate::{fetch, merge, tree_handler};

/// Logs the 'git push' command with the specified branch, local directory, and remote repository name.
///
/// This function logs the 'git push' command with the provided branch, local directory, and remote repository name
/// to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `branch` - A string representing the name of the branch to push.
/// * `local_dir` - A string representing the path to the local directory.
/// * `remote_repo_name` - An optional string representing the name of the remote repository. If not provided, "origin" is used.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_push(branch: &str, local_dir: &str, remote_repo_name: Option<&str>) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let remote_name = remote_repo_name.unwrap_or("origin");
    let full_message = format!(
        "Command 'git push': Branch '{}', Local Directory '{}', Remote Repository Name '{}', {}",
        branch,
        local_dir,
        remote_name,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Perform a Git pull operation to update a local branch from a remote repository.
///
/// This function executes a Git pull operation, which involves fetching the most recent commits and objects
/// from the remote repository and merging the changes into a local branch. It updates the specified `branch` in
/// the local Git repository located in `local_dir` by synchronizing it with the remote repository. The `remote_repo_name`
/// can be optionally provided to specify the name of the remote repository to pull from, and the `host` identifies
/// the host of the remote repository.
///
/// # Arguments
///
/// * `branch`: The name of the local branch to be updated.
/// * `local_dir`: The path to the local directory containing the Git repository.
/// * `remote_repo_name`: An optional name for the remote repository to pull from. If not provided, "origin" is used.
/// * `host`: The host associated with the remote repository.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
///
pub fn git_pull(
    branch: &str,
    local_dir: &str,
    remote_repo_name: Option<&str>,
    host: &str,
) -> io::Result<()> {
    let result = fetch::git_fetch(remote_repo_name, host, local_dir);
    let git_dir = local_dir.to_string() + "/" + GIT_DIR;

    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error: Could not fetch remote repository\n",
        ));
    }
    let remote_ref = format!(
        "{}/refs/remotes/{}/{}",
        git_dir,
        remote_repo_name.unwrap_or("origin"),
        branch
    );
    let hash = match fs::read_to_string(remote_ref) {
        Ok(hash) => hash.trim().to_string(),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Error: Could not find branch in remotes\n",
            ));
        }
    };
    let head_ref = git_dir.to_string() + "/refs/heads/" + branch;
    if Path::new(&head_ref).exists() {
        let tree = merge::merge_remote_branch(branch, &hash, &git_dir)?;
        tree.create_directories(local_dir, &git_dir)?;
    } else {
        let commit_tree = tree_handler::load_tree_from_commit(&hash, &git_dir)?;
        commit_tree.create_directories(local_dir, &git_dir)?;
    }
    update_heads_from_fetch_head(&git_dir)?;
    log_push(branch, local_dir, remote_repo_name)?;
    Ok(())
}

fn update_heads_from_fetch_head(git_dir: &str) -> io::Result<()> {
    let fetch_head_path = git_dir.to_string() + "/FETCH_HEAD";
    let fetch_head = fetch::FetchHead::load_file(&fetch_head_path)?;
    for entry in fetch_head.get_entries() {
        let branch_file_path = git_dir.to_string() + "/refs/heads/" + &entry.branch_name;
        let mut branch_file = std::fs::File::create(branch_file_path)?;
        branch_file.write_all(entry.commit_hash.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::clone;
    const PORT: &str = "9418";

    #[ignore = "This test only works if the server is running"]
    #[test]
    fn test_pull() {
        let local_dir = env::temp_dir().to_str().unwrap().to_string() + "/test_pull";
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "repo_prueba";
        let host = "localhost";
        let _ = clone::git_clone(&address, remote_repo_name, host, &local_dir);

        let result = super::git_pull("branch", &local_dir, Some(remote_repo_name), host);
        assert_eq!(result.is_ok(), true);
    }
}
