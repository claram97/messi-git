use crate::cat_file;
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
        "tree {tree_hash}\nparent {parent}\nauthor {}\n date: {time}\n\n{message}",
        "user"
    );

    let commit_hash = hash_object::store_string_to_file(&commit_content, directory, "commit")?;

    Ok(commit_hash)
}

/// Creates a new commit file and updates the branch file.
/// the refs/heads/branch_name file will be updated with the new commit hash.
/// The branch file must exist. If it doesn't exist, the function will return an error.
/// The index file must exist. If it doesn't exist, the function will return an error.
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
/// * `git_dir_path` - The path to the git directory.
/// * `message` - The commit message.
/// 
/// ## Returns
///
/// The hash of the new commit.
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
    let mut branch_file = std::fs::File::create(&branch_path)?;
    branch_file.write_all(commit_hash.as_bytes())?;

    Ok(commit_hash)
}

/// Returns the parent hash of the given commit hash.
/// If the commit is not found, it returns an error.
/// 
/// ## Parameters
/// 
/// * `commit_hash` - The hash of the commit that you want the parent of.
/// * `git_dir_path` - The path to the git directory.
/// 
pub fn get_parent_hash(commit_hash: &str, git_dir_path: &str) -> io::Result<String> {
    //Open the commit file
    let commit_file = cat_file::cat_file_return_content(commit_hash, git_dir_path)?;
    //Get the parent hash
    let parent_hash = match commit_file.split("\n").nth(1) {
        Some(parent_hash) => match parent_hash.split(" ").nth(1) {
            Some(parent_hash) => parent_hash,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Parent hash not found")),
        },
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "Parent hash not found")),
    };

    Ok(parent_hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_refs_file(git_dir_path: &str) {
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::create(&refs_path).unwrap();
        refs_file.write_all("hash_del_commit_anterior".as_bytes()).unwrap();
    }

    #[test]
    fn test_hash_in_refs_file() {
        //Edit the refs/heads/main file
        let git_dir_path = "tests/commit/.mgit_test";
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message).unwrap();
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::open(&refs_path).unwrap();
        let mut refs_content = String::new();
        refs_file.read_to_string(&mut refs_content).unwrap();
        assert_eq!(refs_content, commit_hash);
    }

    #[test]
    fn test_commit_parent() {
        //Edit the refs/heads/main file
        let git_dir_path = "tests/commit/.mgit_test";
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, "hash_del_commit_anterior");
    }

    #[test]
    fn head_does_not_exist_returns_error() {
        let git_dir_path = "tests/commit";
        let message = "test commit";
        let result = new_commit(git_dir_path, message);
        assert!(result.is_err());
    }

    #[test]
    fn commits_chained_correctly() {
        let git_dir_path = "tests/commit/.mgit_test";
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_1_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_1_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, "hash_del_commit_anterior");
        let message = "test commit 2";
        let commit_2_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_2_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_1_hash);
        let message = "test commit 3";
        let commit_3_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_3_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_2_hash);
    }

    // #[test]
    // fn chained_commits_messages_are_correct() {
    //     let git_dir_path = "tests/commit/.mgit_test";
    //     reset_refs_file(git_dir_path);
    //     let message = "test commit";
    //     let commit_1_hash = new_commit(git_dir_path, message).unwrap();
    //     let commit_1_content = cat_file::cat_file_return_content(&commit_1_hash, git_dir_path).unwrap();
    //     let message = "test commit 2";
    //     let commit_2_hash = new_commit(git_dir_path, message).unwrap();
    //     let commit_2_content = cat_file::cat_file_return_content(&commit_2_hash, git_dir_path).unwrap();
    //     let message = "test commit 3";
    //     let commit_3_hash = new_commit(git_dir_path, message).unwrap();
    //     let commit_3_content = cat_file::cat_file_return_content(&commit_3_hash, git_dir_path).unwrap();
    //     assert_eq!(commit_1_content.split("\n").last().unwrap(), "test commit");
    //     assert_eq!(commit_2_content.split("\n").last().unwrap(), "test commit 2");
    //     assert_eq!(commit_3_content.split("\n").last().unwrap(), "test commit 3");
    // }
}
