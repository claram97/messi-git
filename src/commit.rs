use crate::cat_file;
use crate::hash_object;
use crate::logger::Logger;
use crate::tree_handler;
use crate::tree_handler::has_tree_changed_since_last_commit;
use crate::utils::get_current_time;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const NO_PARENT: &str = "0000000000000000000000000000000000000000";
const INDEX_FILE_NAME: &str = "index";

/// Logs the 'git commit' command with the specified Git directory, commit message, and Git ignore path.
///
/// This function logs the 'git commit' command with the provided Git directory, commit message, and
/// Git ignore path to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `git_dir_path` - The path to the Git directory.
/// * `message` - The commit message.
/// * `git_ignore_path` - The path to the Git ignore file.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_commit(git_dir_path: &str, message: &str, git_ignore_path: &str) -> io::Result<()> {
    let log_file_path = ".logger_commands.txt";
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git commit': Git Directory '{}', Message '{}', Git Ignore Path '{}', {}",
        git_dir_path,
        message,
        git_ignore_path,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Creates a new commit file.
/// With the given tree hash, parent commit and message. Adds the author and date.
/// If no changes were made, it will not create a new commit and will return an error.
/// If the index file doesn't exist, it will return an error.
/// If the commit file was created successfully, it will return the hash of the new commit.
fn create_new_commit_file(
    directory: &str,
    message: &str,
    parent_commit: &str,
    git_ignore_path: &str,
) -> io::Result<String> {
    let index_path = directory.to_string() + "/" + INDEX_FILE_NAME;
    let commit_tree = tree_handler::build_tree_from_index(&index_path, directory, git_ignore_path)?;
    let (tree_hash, _) = tree_handler::write_tree(&commit_tree, directory)?;

    if !has_tree_changed_since_last_commit(&tree_hash, parent_commit, directory) {
        return Err(io::Error::new(io::ErrorKind::Other, "No changes were made"));
    }

    let time = chrono::Local::now();
    let commit_content = format!(
        "tree {tree_hash}\nparent {parent_commit}\nauthor {} {} {time}\ncommitter {} {} {time}\n\n{message}\0","user", "email@email", "user", "email@email"
    );

    let commit_hash = hash_object::store_string_to_file(&commit_content, directory, "commit")?;
    Ok(commit_hash)
}

/// Retrieves the name of the currently checked-out branch in a Git repository.
///
/// This function reads the contents of the Git repository's "HEAD" file to determine the currently
/// checked-out branch and returns its name as a string. The "HEAD" file typically contains a reference
/// to the branch that is currently active.
///
/// # Arguments
///
/// * `git_dir_path`: A string representing the path to the Git repository's root directory.
///
/// # Returns
///
/// Returns a `Result` containing the name of the currently checked-out branch as a string. In case of success,
/// an `io::Result<String>` is returned.
///
pub fn get_branch_name(git_dir_path: &str) -> io::Result<String> {
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
    let name: Vec<&str> = branch_name.split('\n').collect();
    Ok(name[0].to_string())
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
pub fn new_commit(git_dir_path: &str, message: &str, git_ignore_path: &str) -> io::Result<String> {
    let branch_name = get_branch_name(git_dir_path)?;
    let branch_path = git_dir_path.to_string() + "/refs/heads/" + &branch_name;
    let parent_hash = match std::fs::File::open(&branch_path) {
        Ok(mut file) => {
            let mut parent_hash = String::new();
            file.read_to_string(&mut parent_hash)?;
            parent_hash
        }
        Err(_) => NO_PARENT.to_string(),
    };
    let commit_hash = create_new_commit_file(git_dir_path, message, &parent_hash, git_ignore_path)?;
    let mut branch_file = std::fs::File::create(&branch_path)?;
    branch_file.write_all(commit_hash.as_bytes())?;
    log_commit(git_dir_path, message, git_ignore_path)?;
    Ok(commit_hash)
}

/// Creates a new merge commit. merge commits are special as they have two parents. This function should only be used when merging two branches.
///
/// The commit file will be created with the following format:
/// tree <tree_hash>
/// parent <parent_hash>
/// parent <parent_hash>
/// author <author>
///
/// <message>
///
pub fn new_merge_commit(
    git_dir_path: &str,
    message: &str,
    parent_hash: &str,
    parent_hash2: &str,
    git_ignore_path: &str,
) -> io::Result<String> {
    let index_path = git_dir_path.to_string() + "/" + INDEX_FILE_NAME;
    let commit_tree =
        tree_handler::build_tree_from_index(&index_path, git_dir_path, git_ignore_path)?;
    let (tree_hash, _) = tree_handler::write_tree(&commit_tree, git_dir_path)?;
    let time = chrono::Local::now();
    let commit_content = format!("tree {tree_hash}\nparent {parent_hash}\nparent {parent_hash2}\nauthor {} {} {time}\ncommitter {} {} {time}\n\n{message}\0", "user", "email@email", "user", "email@email"
    );
    let commit_hash = hash_object::store_string_to_file(&commit_content, git_dir_path, "commit")?;
    let branch_name = get_branch_name(git_dir_path)?;
    let branch_path = git_dir_path.to_string() + "/refs/heads/" + &branch_name;
    let mut branch_file = std::fs::File::create(branch_path)?;
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
    let commit_file = cat_file::cat_file_return_content(commit_hash, git_dir_path)?;
    let parent_hash: &str = match commit_file.split('\n').nth(1) {
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

pub fn get_commit_message(commit_hash: &str, git_dir_path: &str) -> io::Result<String> {
    let commit_file = cat_file::cat_file_return_content(commit_hash, git_dir_path)?;
    let message: &str = match commit_file.split('\n').nth(5) {
        Some(message) => message,
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "Message not found")),
    };
    Ok(message.to_string())
}
/// Reads and returns the commit hash referred to by the HEAD reference in a Git repository.
///
/// This function reads the contents of the Git repository's "HEAD" file to determine the commit hash
/// to which it is currently referring. The "HEAD" file may contain a reference to a branch or a direct
/// commit hash. This function resolves the reference and returns the associated commit hash as a string.
///
/// # Arguments
///
/// * `git_dir`: A string representing the path to the Git repository's root directory.
///
/// # Returns
///
/// Returns a `Result` containing the commit hash as a string. In case of success, an `io::Result<String>` is returned.
///
pub fn read_head_commit_hash(git_dir: &str) -> io::Result<String> {
    let head_path = format!("{}/HEAD", git_dir);
    let head_content = fs::read_to_string(head_path)?;
    let last_commit_ref = head_content.trim().split(": ").last();

    match last_commit_ref {
        Some(refs) => {
            let heads_path = format!("{}/{}", git_dir, refs);
            if Path::new(&heads_path).exists() {
                fs::read_to_string(heads_path)
            } else {
                Ok(refs.to_string())
            }
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error in head file",
        )),
    }
}

#[cfg(test)]
mod tests {
    fn create_git_dir(git_dir_path: &str) {
        let _ = std::fs::remove_dir_all(git_dir_path);
        let _ = std::fs::create_dir_all(git_dir_path);
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/refs");
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/refs/heads");
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/objects");
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/logs");
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/logs/refs");
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/logs/refs/heads");
        let mut head_file = std::fs::File::create(git_dir_path.to_string() + "/HEAD").unwrap();
        head_file
            .write_all("ref: refs/heads/main".as_bytes())
            .unwrap();

        let mut refs_file =
            std::fs::File::create(git_dir_path.to_string() + "/refs/heads/main").unwrap();
        refs_file
            .write_all("000008507513fcffffffb8914504defeeb800000".as_bytes())
            .unwrap();

        let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
        let index_file_content = "00000855bce90795f20fffff5242cc9235000000 probando.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 src/probando.c\n000008afba902111fffffa8ebcc70522a3e00000 src/pruebita.c\n00000128d8c22fc69fffff0d9620ab896b500000 src/prueba/prueba.c";
        index_file.write_all(index_file_content.as_bytes()).unwrap();
    }
    use super::*;

    fn reset_refs_file(git_dir_path: &str) {
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::create(&refs_path).unwrap();
        refs_file
            .write_all("000008507513fcffffffb8914504defeeb800000".as_bytes())
            .unwrap();
    }

    #[test]
    fn test_hash_in_refs_file() {
        let git_dir_path = "tests/commit/.mgit_test";
        create_git_dir(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message, "").unwrap();
        let refs_path = git_dir_path.to_string() + "/refs/heads/main";
        let mut refs_file = std::fs::File::open(&refs_path).unwrap();
        let mut refs_content = String::new();
        refs_file.read_to_string(&mut refs_content).unwrap();
        assert_eq!(refs_content, commit_hash);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn no_commit_made_if_no_changes() {
        let git_dir_path = "tests/commit/.mgit_test6";
        create_git_dir(git_dir_path);
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message, "");
        let message = "test commit 2";
        let commit_hash2 = new_commit(git_dir_path, message, "");
        assert!(commit_hash.is_ok());
        assert!(commit_hash2.is_err());
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn test_commit_parent_is_correct() {
        let git_dir_path: &str = "tests/commit/.mgit_test1";
        create_git_dir(git_dir_path);
        let refs_dir = git_dir_path.to_string() + "/refs/heads/main";
        let mut ref_actual = std::fs::File::open(&refs_dir).unwrap();
        let mut ref_actual_content = String::new();
        ref_actual.read_to_string(&mut ref_actual_content).unwrap();
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message, "").unwrap();
        let parent_hash = get_parent_hash(&commit_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, ref_actual_content);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn head_does_not_exist_returns_error() {
        let git_dir_path = "tests/commit";
        let message = "test commit";
        let result = new_commit(git_dir_path, message, "");
        assert!(result.is_err());
    }

    #[test]
    fn commits_chained_correctly() {
        let git_dir_path = "tests/commit/.mgit_test2";
        create_git_dir(git_dir_path);
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_1_hash = new_commit(git_dir_path, message, "").unwrap();
        let parent_hash = get_parent_hash(&commit_1_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, "000008507513fcffffffb8914504defeeb800000");
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let message = "test commit 2";
        let commit_2_hash = new_commit(git_dir_path, message, "").unwrap();
        let parent_hash = get_parent_hash(&commit_2_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_1_hash);

        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\n3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/prueba/prueba3.c".as_bytes())
            .unwrap();
        let message = "test commit 3";
        let commit_3_hash = new_commit(git_dir_path, message, "").unwrap();
        let parent_hash = get_parent_hash(&commit_3_hash, git_dir_path).unwrap();
        assert_eq!(parent_hash, commit_2_hash);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn chained_commits_messages_are_correct() {
        let git_dir_path = "tests/commit/.mgit_test3";
        create_git_dir(git_dir_path);
        reset_refs_file(git_dir_path);
        let message = "test commit";
        let commit_1_hash = new_commit(git_dir_path, message, "").unwrap();
        let commit_1_content =
            cat_file::cat_file_return_content(&commit_1_hash, git_dir_path).unwrap();

        let message = "test commit 2";
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\n0894f78e615131459e43d258070b5540081f1d82 src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let commit_2_hash = new_commit(git_dir_path, message, "").unwrap();
        let commit_2_content =
            cat_file::cat_file_return_content(&commit_2_hash, git_dir_path).unwrap();
        let message = "test commit 3";
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\n85628bead31d2c14e4a56113e524eab2ccff22c9 src/prueba/prueba3.c".as_bytes())
            .unwrap();
        let commit_3_hash = new_commit(git_dir_path, message, "").unwrap();
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
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn test_get_message_works_correctly() {
        let git_dir_path: &str = "tests/commit/.mgit_test_message";
        create_git_dir(git_dir_path);
        let refs_dir = git_dir_path.to_string() + "/refs/heads/main";
        let mut ref_actual = std::fs::File::open(&refs_dir).unwrap();
        let mut ref_actual_content = String::new();
        ref_actual.read_to_string(&mut ref_actual_content).unwrap();
        let message = "test commit";
        let commit_hash = new_commit(git_dir_path, message, "").unwrap();
        let commit_message = get_commit_message(&commit_hash, git_dir_path).unwrap();
        assert_eq!(commit_message, message);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }
}
