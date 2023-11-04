use std::{io::{self, Read}, path::{PathBuf, Path}};

use crate::{client::Client, tree_handler, init};

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


pub fn git_clone(
    repo_url: &str,
    remote_repo_name: &str,
    host: &str,
    local_dir: &str,
) -> io::Result<()> {

    init::git_init(local_dir, "master", None)?;
    let local_git_dir = local_dir.to_string() + "/.mgit";
    //Establish the connection
    let mut client = Client::new(&repo_url, remote_repo_name, host);
    let refs = client.get_refs()?;
    let clean_refs = refs
        .iter()
        .map(|x| match x.split('/').last() {
            Some(string) => string.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();

    for server_ref in clean_refs {
        let result = client.upload_pack(&server_ref, &local_git_dir, "origin");
        println!("{:?}", result);
    }

    //Now that we have the data, checkout the HEAD commit to the working dir
    let default_branch_commit = get_default_branch_commit(&local_git_dir)?;
    let commit_tree = tree_handler::load_tree_from_commit(&default_branch_commit, &local_git_dir)?;
    println!("{}", default_branch_commit);
    let path_buf = PathBuf::from(&local_git_dir);
    let binding = PathBuf::from("");
    let parent_dir: &Path = match path_buf.parent() {
        Some(value) => value,
        None => &binding
    };
    let parent_dir = match parent_dir.to_str() {
        Some(value) => value,
        None => ""
    };
    commit_tree.create_directories(parent_dir, &local_git_dir)?;

    Ok(())
}
