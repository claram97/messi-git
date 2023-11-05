use std::io::{self, Write};

use crate::{fetch, merge};

/// Checks if a branch exists in the local Git repository's references.
///
/// This function determines whether a specific branch, identified by its name, exists in the local Git repository's references.
/// It checks if a corresponding file for the branch is present in the ".mgit/refs/heads/" directory.
///
/// # Arguments
///
/// * `branch`: The name of the branch to check for existence.
/// * `local_dir`: The path to the local directory containing the Git repository.
///
/// # Returns
///
/// Returns `true` if the branch exists in the references, and `false` otherwise.
///
fn branch_is_in_refs(branch: &str, local_dir: &str) -> bool {
    let path = local_dir.to_string() + "/.mgit/refs/heads/" + branch;
    let result = std::fs::File::open(path);
    result.is_ok()
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
    let git_dir = local_dir.to_string() + "/.mgit";

    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error: Could not fetch remote repository",
        ));
    }
    let fetch_head_path = git_dir.to_string() + "/FETCH_HEAD";
    let fetch_head = fetch::FetchHead::load_file(&fetch_head_path)?;

    let branch_remotes = match fetch_head.get_branch_entry(branch) {
        Some(branch_remotes) => branch_remotes,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error: Could not find branch in FETCH_HEAD",
            ));
        }
    };

    if !branch_is_in_refs(branch, local_dir) {
        let branch_file_path = git_dir.to_string() + "/refs/heads/" + branch;
        let mut branch_file = std::fs::File::create(branch_file_path)?;
        branch_file.write_all(branch_remotes.commit_hash.as_bytes())?;
    }

    let hash = branch_remotes.commit_hash.clone();
    merge::merge_remote_branch(branch, &hash, &git_dir)?;
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
