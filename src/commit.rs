use crate::hash_object;
use crate::tree_handler;
use std::io;
use std::io::Read;
use std::io::Write;


const NO_PARENT: &str = "0000000000000000000000000000000000000000";
const INDEX_FILE_NAME: &str = "index";

fn create_new_commit_file(
    directory: &str,
    message: &str,
    parent: &str,
) -> io::Result<String> {
    let index_path = directory.to_string() + "/" + INDEX_FILE_NAME;
    let commit_tree = tree_handler::build_tree_from_index(&index_path, directory)?;
    let (tree_hash, _) = tree_handler::write_tree(&commit_tree, directory)?;
    let time = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let commit_content = format!(
        "tree {tree_hash}\nparent {parent}\nauthor {}\n date: {time}\n\n {message}",
        "user"
    );

    let commit_hash = hash_object::store_string_to_file(&commit_content, directory, "commit")?;

    Ok(commit_hash)
}

/// Creates a new commit file and updates the branch file.
/// the refs/heads/branch_name file will be updated with the new commit hash.
/// The branch file must exist. If it doesn't exist, the function will return an error.
/// 
/// The commit file will be created with the following format:
/// tree <tree_hash>
/// parent <parent_hash>
/// author <author>
/// date: <date>
/// 
/// <message>
/// 
/// ## Parameters
/// 
/// 
pub fn new_commit(git_dir_path: &str, message: &str) -> io::Result<String> {
    let head_path = git_dir_path.to_string() + "/HEAD";
    let mut head_file = std::fs::File::open(&head_path)?;
    let mut head_content = String::new();
    head_file.read_to_string(&mut head_content)?;
    
    let branch_name = match head_content.split("/").last() {
        Some(branch_name) => branch_name,
        None => return Err(io::Error::new(io::ErrorKind::NotFound , "HEAD file is empty")),
    };

    let branch_path = git_dir_path.to_string() + "/refs/heads/" + branch_name;
    let mut branch_file = std::fs::File::open(&branch_path)?;
    let mut parent_hash = String::new();
    branch_file.read_to_string(&mut parent_hash)?;
    if parent_hash == "" {
        parent_hash = NO_PARENT.to_string();
    }
    let commit_hash = create_new_commit_file(git_dir_path, message, &parent_hash)?;
    branch_file.write_all(commit_hash.as_bytes())?;

    Ok(commit_hash)
}
