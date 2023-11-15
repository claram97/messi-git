use crate::{client::Client, config};
use std::io;

/// Pushes the specified branch to the remote repository.
///
/// This function loads the Git configuration file, retrieves the remote URL, and uses it to
/// create a Git client. It then calls the `receive_pack` method of the client to push the branch
/// to the remote repository.
///
/// # Arguments
///
/// * `branch` - The name of the branch to be pushed.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// A Result indicating success or an io::Error if an issue occurs during the push operation.
///
pub fn git_push(branch: &str, git_dir: &str) -> io::Result<()> {
    let config_file = config::Config::load(git_dir)?;
    let remote_name = "origin";
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
    let mut client = Client::new(address, repo_name, "localhost");
    client.receive_pack(branch, git_dir)
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        io::{self, Write},
        path::PathBuf,
    };

    use crate::{add, branch, checkout, clone, commit};
    const PORT: &str = "9418";
    #[ignore = "This test only works if the server is running"]
    #[test]
    fn test_push() {
        let local_dir = env::temp_dir().to_str().unwrap().to_string() + "/test_push";
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "prueba_clonar";
        let host = "localhost";
        let git_dir_path = local_dir.clone() + "/.mgit";
        let _ = clone::git_clone(&address, remote_repo_name, host, &local_dir);
        let _ = branch::create_new_branch(&git_dir_path, "branch", &mut io::stdout());
        let _ = checkout::checkout_branch(&PathBuf::from(&git_dir_path), &local_dir, "branch");
        //Create two new files to push
        let file_path = local_dir.clone() + "/test_file.txt";
        let file_path2 = local_dir.clone() + "/test_file2.txt";
        let mut file = std::fs::File::create(&file_path).unwrap();
        let mut file2 = std::fs::File::create(&file_path2).unwrap();
        file.write_all(b"test").unwrap();
        file2.write_all(b"test2").unwrap();
        let index_path = local_dir.clone() + "/.mgit/index";
        //Add the files to the index
        let _ = add::add(&file_path, &index_path, &git_dir_path, "", None);
        let _ = add::add(&file_path2, &index_path, &git_dir_path, "", None);
        //Commit the files
        let commit_message = "Test commit".to_string();
        let result_commit = commit::new_commit(&git_dir_path, &commit_message, "");
        let result = super::git_push("branch", &git_dir_path);
        assert!(result_commit.is_ok());
        assert!(result.is_ok());
    }
}
