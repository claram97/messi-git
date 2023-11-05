use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::{client::Client, init, tree_handler, branch, config};

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

fn get_clean_refs(refs: Vec<String>) -> Vec<String> {
    let clean_refs = refs
        .iter()
        .map(|x| match x.split('/').last() {
            Some(string) => string.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();
    clean_refs
}

fn get_parent_dir(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    let parent = match path_buf.parent() {
        Some(parent) => parent,
        None => Path::new(""),
    };
    let parent_dir = match parent.to_str() {
        Some(parent_dir) => parent_dir,
        None => "",
    };
    parent_dir.to_string()
}

pub fn git_clone(
    remote_repo_url: &str,
    remote_repo_name: &str,
    host: &str,
    local_dir: &str,
) -> io::Result<()> {
    init::git_init(local_dir, "master", None)?;
    let local_git_dir = local_dir.to_string() + "/.mgit";
    let mut client = Client::new(&remote_repo_url, remote_repo_name, host);
    let refs = client.get_refs()?;
    let clean_refs = get_clean_refs(refs);

    for server_ref in clean_refs {
        let result = client.upload_pack(&server_ref, &local_git_dir, "origin");
        if result.is_err() {
            println!("Error: {:?}", result);
        }
    }
    let default_branch_commit = get_default_branch_commit(&local_git_dir)?;
    let commit_tree = tree_handler::load_tree_from_commit(&default_branch_commit, &local_git_dir)?;
    let parent_dir = &local_dir;
    let branch_file_path = local_git_dir.to_string() + "/refs/heads/master";
    let mut branch_file = std::fs::File::create(branch_file_path)?;
    branch_file.write_all(default_branch_commit.as_bytes())?;
    commit_tree.create_directories(&parent_dir, &local_git_dir)?;
    let index_path = local_git_dir.to_string() + "/index";
    let gitignore_path = parent_dir.to_string() + "/.gitignore";
    commit_tree.build_index_file_from_tree(&index_path, &local_git_dir, &gitignore_path)?;
    let mut config_file = config::Config::load(&local_git_dir)?;    
    config_file.add_remote(remote_repo_name.to_string(), remote_repo_url.to_string(), remote_repo_url.to_string(), &mut io::stdout())?;
    config_file.add_branch("master".to_string(), "origin".to_string(), "/refs/heads/master".to_string(), &mut io::stdout())?;
    Ok(())
}

pub fn git_fetch(remote_repo_url: &str, remote_repo_name: &str, host: &str, local_dir: &str) -> io::Result<()> {
    let local_git_dir = local_dir.to_string() + "/.mgit";
    let mut client = Client::new(&remote_repo_url, remote_repo_name, host);
    let refs = client.get_refs()?;
    let clean_refs = get_clean_refs(refs);

    for server_ref in clean_refs {
        let result = client.upload_pack(&server_ref, &local_git_dir, "origin");
        if result.is_err() {
            println!("Error: {:?}", result);
        }
    }
    Ok(())
}

