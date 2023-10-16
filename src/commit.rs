use crate::hash_object;
use std::io;

use crate::tree_handler;


const INDEX_FILE_NAME: &str = "index";

pub fn create_new_commit_file(directory: &str, message: &str, parent: Option<&str>) -> io::Result<String>{
    let index_path = directory.to_string() + "/" + INDEX_FILE_NAME;
    let commit_tree = tree_handler::build_tree_from_index(&index_path, directory)?;
    let (tree_hash, _) = tree_handler::write_tree(&commit_tree, directory)?;
    let time = chrono::Local::now().format("%s").to_string();
    let commit_parent = match parent {
        Some(parent) => parent,
        None => "0000000000000000000000000000000000000000000",
    };
    let commit_content = format!("tree {tree_hash}\nparent {commit_parent}\nauthor {}\n date: {time}\n\n {message}", "user");
    
    let commit_hash = hash_object::store_string_to_file(&commit_content, directory, "commit")?;

    Ok(commit_hash)
}
