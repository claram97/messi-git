use crate::cat_file;
use crate::hash_object;
use crate::tree_handler;
use crate::tree_handler::has_tree_changed_since_last_commit;
use std::io;
use std::io::Read;
use std::io::Write;

const NO_PARENT: &str = "0000000000000000000000000000000000000000";
const INDEX_FILE_NAME: &str = "index";

/// Creates a new commit file.
/// With the given tree hash, parent commit and message. Adds the author and date.
/// If no changes were made, it will not create a new commit and will return an error.
/// If the index file doesn't exist, it will return an error.
/// If the commit file was created successfully, it will return the hash of the new commit.
fn create_new_commit_file(
    directory: &str,
    message: &str,
    parent_commit: &str,
) -> io::Result<String> {
    let index_path = directory.to_string() + "/" + INDEX_FILE_NAME;
    let commit_tree = tree_handler::build_tree_from_index(&index_path, directory)?;
    let (tree_hash, _) = tree_handler::write_tree(&commit_tree, directory)?;

    if !has_tree_changed_since_last_commit(&tree_hash, parent_commit, directory) {
        return Err(io::Error::new(io::ErrorKind::Other, "No changes were made"));
    }

    let time = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let commit_content = format!(
        "tree {tree_hash}\nparent {parent_commit}\nauthor {}\ndate: {time}\n\n{message}\0",
        "user"
    );

    let commit_hash = hash_object::store_string_to_file(&commit_content, directory, "commit")?;
    Ok(commit_hash)
}

/// Creates a new commit file and updates the branch file.
/// the refs/heads/branch_name file will be updated with the new commit hash.
/// If the branch file doesn't exist, it will be created.
/// The index file must exist. If it doesn't exist, the function will return an error.
///
/// If no changes were made, it will not create a new commit and will return an error.
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
    let mut head_file = std::fs::File::open(head_path)?;
    let mut head_content = String::new();
    head_file.read_to_string(&mut head_content)?;

    let branch_name = match head_content.split('/').last() {
        Some(branch_name) => branch_name,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HEAD file is empty",
            ))
        }
    };

    let branch_path = git_dir_path.to_string() + "/refs/heads/" + branch_name;
    match std::fs::File::open(&branch_path) {
        Ok(mut file) => {
            let mut parent_hash = String::new();
            file.read_to_string(&mut parent_hash)?;
            let commit_hash = create_new_commit_file(git_dir_path, message, &parent_hash)?;
            let mut branch_file = std::fs::File::create(&branch_path)?;
            branch_file.write_all(commit_hash.as_bytes())?;
            Ok(commit_hash)
        }
        Err(_) => {
            let commit_hash = create_new_commit_file(git_dir_path, message, NO_PARENT)?;
            let mut branch_file = std::fs::File::create(&branch_path)?;
            branch_file.write_all(commit_hash.as_bytes())?;
            Ok(commit_hash)
        }
    }
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
    let parent_hash = match commit_file.split('\n').nth(1) {
        Some(parent_hash) => match parent_hash.split(' ').nth(1) {
            Some(parent_hash) => parent_hash,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Parent hash not found",
                ))
            }
        },
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Parent hash not found",
            ))
        }
    };

    Ok(parent_hash.to_string())
}

#[cfg(test)]
mod tests {
    fn rebuild_git_dir(git_dir_path: &str) {
        let _ = std::fs::remove_dir_all(git_dir_path);
        let _ = std::fs::create_dir(git_dir_path);
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/refs");
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/refs/heads");
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/objects");
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs");
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs/refs");
        let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs/refs/heads");
        let mut head_file = std::fs::File::create(git_dir_path.to_string() + "/HEAD").unwrap();
        head_file
            .write_all("ref: refs/heads/main".as_bytes())
            .unwrap();

        //Create the refs/heads/main file
        let mut refs_file =
            std::fs::File::create(git_dir_path.to_string() + "/refs/heads/main").unwrap();
        refs_file
            .write_all("hash_del_commit_anterior".as_bytes())
            .unwrap();

        //Create the index file
        let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
        let index_file_content = "hashhashhashhash1 probando.txt\nhashhashhashhash2 src/probando.c\nhashhashhashhash3 src/pruebita.c\nhashhashhashhash4 src/prueba/prueba.c";
        index_file.write_all(index_file_content.as_bytes()).unwrap();
    }
    use super::*;

    fn reset_refs_file(git_dir_path: &str) {
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::create(&refs_path).unwrap();
        refs_file
            .write_all("hash_del_commit_anterior".as_bytes())
            .unwrap();
    }

    #[test]
    fn test_hash_in_refs_file() {
        let git_dir_path = "tests/commit/.mgit_test";
        rebuild_git_dir(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message).unwrap();
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::open(&refs_path).unwrap();
        let mut refs_content = String::new();
        refs_file.read_to_string(&mut refs_content).unwrap();
        assert_eq!(refs_content, commit_hash);
    }

    #[test]
    fn no_commit_made_if_no_changes() {
        let git_dir_path = "tests/commit/.mgit_test6";
        rebuild_git_dir(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message);
        let message = "test commit 2";
        let commit_hash2 = new_commit(git_dir_path, message);
        assert!(commit_hash.is_ok());
        assert!(commit_hash2.is_err());
    }

    #[test]
    fn test_commit_parent_is_correct() {
        let git_dir_path: &str = "tests/commit/.mgit_test1";
        rebuild_git_dir(git_dir_path);
        let refs_dir = git_dir_path.to_string() + "/refs/heads/main";
        let mut ref_actual = std::fs::File::open(&refs_dir).unwrap();
        let mut ref_actual_content = String::new();
        ref_actual.read_to_string(&mut ref_actual_content).unwrap();
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, ref_actual_content);
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
        let git_dir_path = "tests/commit/.mgit_test2";
        rebuild_git_dir(git_dir_path);
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_1_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_1_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, "hash_del_commit_anterior");
        //Append a "change" to the index file
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\nhashhashhashhash5 src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let message = "test commit 2";
        let commit_2_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_2_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_1_hash);

        //Append a "change" to the index file
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\nhashhashhashhash6 src/prueba/prueba3.c".as_bytes())
            .unwrap();
        let message = "test commit 3";
        let commit_3_hash = new_commit(git_dir_path, message).unwrap();
        let parent_hash = get_parent_hash(&commit_3_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_2_hash);
    }

    #[test]
    fn chained_commits_messages_are_correct() {
        let git_dir_path = "tests/commit/.mgit_test3";
        rebuild_git_dir(git_dir_path);
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_1_hash = new_commit(git_dir_path, message).unwrap();
        let commit_1_content =
            cat_file::cat_file_return_content(&commit_1_hash, git_dir_path).unwrap();

        let message = "test commit 2";
        //Append a "change" to the index file
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\nhashhashhashhash5 src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let commit_2_hash = new_commit(git_dir_path, message).unwrap();
        let commit_2_content =
            cat_file::cat_file_return_content(&commit_2_hash, git_dir_path).unwrap();
        let message = "test commit 3";
        //Append a "change" to the index file
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\nhashhashhashhash6 src/prueba/prueba3.c".as_bytes())
            .unwrap();
        let commit_3_hash = new_commit(git_dir_path, message).unwrap();
        let commit_3_content =
            cat_file::cat_file_return_content(&commit_3_hash, git_dir_path).unwrap();
        assert_eq!(commit_1_content.split("\n").last().unwrap(), "test commit");
        assert_eq!(
            commit_2_content.split("\n").last().unwrap(),
            "test commit 2"
        );
        assert_eq!(
            commit_3_content.split("\n").last().unwrap(),
            "test commit 3"
        );
    }
}
