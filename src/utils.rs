use std::{io, path::PathBuf};

use crate::commit;

/// Recursively searches for a directory named "name_of_git_directory" in the file system
/// starting from the location specified by "current_dir."
///
/// # Arguments
///
/// * `current_dir`: A mutable reference to a `PathBuf` representing the initial location from which the search begins.
/// * `name_of_git_directory`: The name of the directory being sought.
///
/// # Returns
///
/// This function returns an `Option<String>` containing the path to the found directory as a string if it is found.
/// If the directory is not found, it returns `None`.
pub fn find_git_directory(
    current_dir: &mut PathBuf,
    name_of_git_directory: &str,
) -> Option<String> {
    loop {
        let git_dir = current_dir.join(name_of_git_directory);
        if git_dir.exists() && git_dir.is_dir() {
            return Some(git_dir.display().to_string());
        }

        if !current_dir.pop() {
            break;
        }
    }
    None
}

pub fn get_branch_commit_history_with_messages(
    commit_hash: &str,
    git_dir: &str,
) -> io::Result<Vec<(String, String)>> {
    let mut parents: Vec<(String, String)> = Vec::new();
    let commit_message: String = commit::get_commit_message(commit_hash, git_dir)?;
    parents.push((commit_hash.to_string(), commit_message.to_string()));
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        println!("{}", parent);
        let commit_message = match commit::get_commit_message(&parent, git_dir) {
            Ok(message) => message,
            Err(_) => break,
        };
        parents.push((parent.clone(), commit_message.to_string()));
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

#[cfg(test)]
mod tests {
    use super::*;
    const NAME_OF_GIT_DIRECTORY: &str = ".test_git";

    #[test]
    fn find_git_directory_returns_none_when_no_git_directory_is_found() {
        let mut current_dir = PathBuf::from("tests/utils/empty");
        let git_directory_name = NAME_OF_GIT_DIRECTORY;

        assert_eq!(
            find_git_directory(&mut current_dir, git_directory_name),
            None
        );
    }

    #[test]
    fn find_git_directory_returns_path_to_git_directory_when_found() {
        let mut current_dir = PathBuf::from("tests/utils/not_empty");
        let git_directory_name = NAME_OF_GIT_DIRECTORY;

        let expected_path = "tests/utils/not_empty/.test_git";
        let expected_path = expected_path.to_string();

        assert_eq!(
            find_git_directory(&mut current_dir, git_directory_name),
            Some(expected_path)
        );
    }
}
