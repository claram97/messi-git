use crate::configuration::{
    GIT_DIR, INITIAL_BRANCH, INITIAL_BRANCH_REF, LOGGER_COMMANDS_FILE, REMOTE,
};
use crate::logger::Logger;
use crate::utils::get_current_time;
use crate::{client::Client, config, init, tree_handler};
use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
};

/// Logs the 'git clone' command with the specified repository URL and destination.
///
/// This function logs the 'git clone' command with the provided repository URL and destination to a
/// file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `repo_url` - The URL of the repository to clone.
/// * `destination` - The destination path for the cloned repository.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_clone(repo_url: &str, destination: &str) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git clone': Repository URL '{}', Destination '{}', {}",
        repo_url,
        destination,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Retrieves the commit hash of the default branch from the local Git repository.
///
/// This function takes the path to the local Git repository directory and looks for the "origin/master"
/// reference to obtain the commit hash of the "master" branch.
///
/// # Arguments
///
/// * `local_git_dir`: The path to the local Git repository directory.
///
/// # Returns
///
/// Returns a `Result` containing the commit hash of the "master" branch in case of success,
/// or an error in case an issue occurs during the operation.
///

//Ver de dónde sale la default branch
fn get_default_branch_commit(local_git_dir: &str) -> io::Result<String> {
    let path_to_file = local_git_dir.to_string() + "/refs/remotes/origin/master";
    println!("{}", path_to_file);
    let mut branch_file = std::fs::File::open(path_to_file)?;
    let mut branch_content = String::new();
    branch_file.read_to_string(&mut branch_content)?;
    let nombre: Vec<&str> = branch_content.split('\n').collect();
    let path_final = nombre[0];
    Ok(path_final.to_string())
}

/// Extracts the last component of each reference from a list of references.
///
/// This function takes a vector of reference strings and extracts the last component of each reference
/// by splitting the string at '/' and taking the last part. It returns a new vector containing only
/// the last components of the references.
///
/// # Arguments
///
/// * `refs`: A vector of reference strings to process.
///
/// # Returns
///
/// Returns a new vector containing only the last components of the input references.
///
fn get_clean_refs(refs: HashMap<String, String>) -> Vec<String> {
    let clean_refs = refs
        .iter()
        .map(|x| match x.0.split('/').last() {
            Some(string) => string.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();
    clean_refs
}

/// Creates a working directory based on the default branch commit of a local Git repository.
///
/// # Arguments
///
/// * `local_git_dir` - A string specifying the path to the local Git directory.
/// * `working_dir` - A string specifying the path to the desired working directory.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If there is an issue while creating or writing to files.
/// - If there is an issue while loading the default branch commit or the commit tree.
/// - If there is an issue while creating directories or building the index file.
///
fn create_working_dir(local_git_dir: &str, working_dir: &str) -> io::Result<()> {
    let default_branch_commit = get_default_branch_commit(local_git_dir)?;
    let commit_tree = tree_handler::load_tree_from_commit(&default_branch_commit, local_git_dir)?;
    let branch_file_path = local_git_dir.to_string() + "/refs/heads/master";
    let mut branch_file = std::fs::File::create(branch_file_path)?;
    branch_file.write_all(default_branch_commit.as_bytes())?;
    commit_tree.create_directories(working_dir, local_git_dir)?;
    let index_path = local_git_dir.to_string() + "/index";
    let gitignore_path = working_dir.to_string() + "/.gitignore";
    let index =
        commit_tree.build_index_file_from_tree(&index_path, local_git_dir, &gitignore_path)?;
    index.write_file()?;
    Ok(())
}

/// Create the refs/remotes/origin/dir
fn create_remote_dir(local_git_dir: &str) -> io::Result<()> {
    let path_to_file = local_git_dir.to_string() + "/refs/remotes/origin/";
    fs::create_dir_all(path_to_file)?;
    Ok(())
}

/// Clone a remote Git repository into a local directory using a custom Git client.
///
/// This function clones a remote Git repository located at `remote_repo_url` into the local directory
/// specified by `local_dir`. It initializes a new Git repository in the local directory, fetches the
/// references (branches and tags) from the remote repository, and creates a local copy of the default
/// branch. The remote repository is identified by `remote_repo_name` and `host`. The function utilizes
/// a custom Git client to perform the cloning operation.
///
/// # Arguments
///
/// * `remote_repo_url`: The URL of the remote Git repository to clone.
/// * `remote_repo_name`: The name of the remote Git repository.
/// * `host`: The host associated with the remote Git repository.
/// * `local_dir`: The path to the local directory where the repository will be cloned.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
///
pub fn git_clone(
    remote_repo_url: &str,
    remote_repo_name: &str,
    host: &str,
    working_dir: &str,
) -> io::Result<()> {
    log_clone(remote_repo_url, working_dir)?;

    init::git_init(working_dir, GIT_DIR, INITIAL_BRANCH, None)?;

    let local_git_dir = working_dir.to_string() + "/" + GIT_DIR;
    let mut client = Client::new(remote_repo_url, remote_repo_name, host);
    let refs = client.get_server_refs()?;
    let clean_refs = get_clean_refs(refs);
    create_remote_dir(&local_git_dir)?;
    let _ = client.upload_pack(clean_refs, &local_git_dir, REMOTE);
    create_working_dir(&local_git_dir, working_dir)?;
    let mut config_file = config::Config::load(&local_git_dir)?;
    config_file.add_remote(
        REMOTE.to_string(),
        remote_repo_url.to_string() + "/" + remote_repo_name,
        remote_repo_url.to_string(),
        &mut io::stdout(),
    )?;
    config_file.add_branch(
        INITIAL_BRANCH.to_string(),
        REMOTE.to_string(),
        INITIAL_BRANCH_REF.to_string(),
        &mut io::stdout(),
    )?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use std::env;

    const PORT: &str = "9418";

    #[ignore = "This test only makes sense if a server is running"]
    #[test]
    fn test_git_clone() {
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "repo_prueba";
        let host = "localhost";
        let mut tmp_dir = env::temp_dir();
        tmp_dir.push("test_clone");
        let _ = std::fs::create_dir(tmp_dir.to_str().unwrap());

        let result = super::git_clone(&address, remote_repo_name, host, tmp_dir.to_str().unwrap());

        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(tmp_dir.to_str().unwrap());
    }
}
