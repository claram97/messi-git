use std::io;

use crate::{client::Client, config};



pub fn git_push(branch: &str, git_dir: &str) -> io::Result<()>{
    let config_file = config::Config::load(git_dir)?;

    let remote_name = match config_file.get_branch_remote_name(branch) {
        Some(name) => name,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se ha encontrado el nombre del repositorio remoto.\n",
            ));
        }
    };
    let remote_repo_url = config_file.get_url(&remote_name, &mut io::stdout())?;
    let host = match remote_repo_url.split_once(':') {
        Some((host, _)) => host,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se ha encontrado el host del repositorio remoto.\n",
            ));
        }
    };
    let mut client = Client::new(&remote_repo_url, &remote_name, host);
    let result = client.receive_pack(branch, git_dir);
    result
}

#[cfg(test)]
mod tests {
    use std::{env, io::Write};

    use crate::{clone, add, commit};
    const PORT: &str = "9418";
    #[ignore = "This test only works if the server is running"]
    #[test]
    fn test_push() {
        let local_dir = env::temp_dir().to_str().unwrap().to_string() + "/test_push";
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "repo_prueba";
        let host = "localhost";
        let _ = clone::git_clone(&address, remote_repo_name, host, &local_dir);
        //Create two new files to push
        let file_path = local_dir.clone() + "/test_file.txt";
        let file_path2 = local_dir.clone() + "/test_file2.txt";
        let mut file = std::fs::File::create(&file_path).unwrap();
        let mut file2 = std::fs::File::create(&file_path2).unwrap();
        file.write_all(b"test").unwrap();
        file2.write_all(b"test2").unwrap();
        let index_path = local_dir.clone() + "/.mgit/index";
        let git_dir_path = local_dir.clone() + "/.mgit";
        //Add the files to the index
        add::add(&file_path, &index_path, &git_dir_path, "", None);
        add::add(&file_path2, &index_path, &git_dir_path, "", None);
        //Commit the files
        let commit_message = "Test commit".to_string();
        let result_commit = commit::new_commit(&git_dir_path, &commit_message, "");
        let result = super::git_push("master",&git_dir_path);
        assert!(result_commit.is_ok());
        assert!(result.is_ok());
    }


}