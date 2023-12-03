use std::{collections::HashSet, io, path::PathBuf};

use chrono::{DateTime, FixedOffset, Offset, Utc};

use crate::{commit, configuration::GIT_DIR};

/// Obtains the path to the Git directory of the current project.
///
/// This function starts from the current directory and traverses upwards until it finds
/// the `.git` directory or the user-configured Git directory. The path to the Git directory
/// is returned as a `String` on success.
///
/// # Returns
///
/// Returns a `Result` containing the path to the Git directory as a `String` on success.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue obtaining the actual directory or locating the Git directory.
///
pub fn obtain_git_dir() -> Result<String, io::Error> {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error obtaining actual directory: {:?}", err);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error obtaining actual directory",
            ));
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
        Some(dir) => dir,
        None => {
            eprintln!("Error obtaining git dir");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error obtaining git dir",
            ));
        }
    };
    Ok(git_dir)
}

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

pub fn get_current_time() -> String {
    use chrono::Local;
    Local::now().to_string()
}

/// Retrieves the commit history of a branch with corresponding commit messages.
///
/// This function takes a commit hash and the Git directory, then traverses the commit
/// history backwards, collecting commit hashes along with their commit messages. The result
/// is a vector of tuples, each containing a commit hash and its associated commit message.
///
/// # Arguments
///
/// * `commit_hash` - The hash of the initial commit to start retrieving the history.
/// * `git_dir` - The path to the Git directory containing the repository information.
///
/// # Returns
///
/// Returns a vector of tuples, where each tuple contains a commit hash and its commit message.
/// The vector represents the commit history of the given branch.
///
pub fn get_branch_commit_history_with_messages(
    commit_hash: &str,
    git_dir: &str,
) -> io::Result<Vec<(String, String)>> {
    let mut parents: Vec<(String, String)> = Vec::new();
    let commit_message: String = commit::get_commit_message(commit_hash, git_dir)?;
    parents.push((commit_hash.to_string(), commit_message.to_string()));
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        let commit_message = match commit::get_commit_message(&parent, git_dir) {
            Ok(message) => message,
            Err(_) => break,
        };
        parents.push((parent.clone(), commit_message.to_string()));
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

/// Get the commit history for a given commit hash in a Git repository.
///
/// This function retrieves the commit history for a specified commit hash by recursively
/// traversing the parent commits until the initial commit (root) is reached. The resulting
/// commit history is returned as a vector of commit hashes in chronological order, starting
/// from the provided commit hash and going back in time.
///
/// # Arguments
///
/// * `commit_hash`: A string representing the commit hash from which to start fetching the history.
/// * `git_dir`: A string representing the path to the Git repository directory.
///
/// # Returns
///
/// Returns a `Result` with a vector of commit hashes if successful. If an error occurs during
/// the retrieval process, it returns an `io::Result` with an error message.
///
pub fn get_branch_commit_history(commit_hash: &str, git_dir: &str) -> io::Result<Vec<String>> {
    let mut parents = Vec::new();
    parents.push(commit_hash.to_string());
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        parents.push(parent.clone());
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

/// Retrieves the commit history of a Git branch until a specified commit hash.
///
/// This function starts from the given `commit_hash` and traverses the parent commits until
/// reaching the commit specified by `until`. The commit history, represented by a vector of commit
/// hashes, is returned on success.
///
/// # Arguments
///
/// - `commit_hash`: The hash of the commit from which to start retrieving the history.
/// - `git_dir`: The path to the Git directory.
/// - `until`: The hash of the commit until which the history should be retrieved.
///
/// # Returns
///
/// Returns a `Result` containing the commit history as a vector of commit hashes on success.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue obtaining commit information or traversing the history.
///
pub fn get_branch_commit_history_until(
    commit_hash: &str,
    git_dir: &str,
    until: &str,
) -> io::Result<Vec<String>> {
    let mut parents = Vec::new();
    parents.push(commit_hash.to_string());
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        if parent == until {
            break;
        }
        parents.push(parent.clone());
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

/// Get the unique commit history for a given commit hash in a Git repository.
///
/// This function retrieves the unique commit history for a specified commit hash by recursively
/// traversing the parent commits until the initial commit (root) is reached. The resulting
/// commit history is returned as a `HashSet` of commit hashes, ensuring uniqueness in the history.
///
/// # Arguments
///
/// * `commit_hash`: A string representing the commit hash from which to start fetching the history.
/// * `git_dir`: A string representing the path to the Git repository directory.
///
/// # Returns
///
/// Returns a `Result` with a `HashSet` of unique commit hashes if successful. If an error occurs
/// during the retrieval process, it returns an `io::Result` with an error message.
///
pub fn get_branch_commit_history_set(
    commit_hash: &str,
    git_dir: &str,
) -> io::Result<HashSet<String>> {
    let mut parents = HashSet::new();
    parents.insert(commit_hash.to_string());
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        parents.insert(parent.clone());
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

/// Get the path to the Git index file in a Git repository.
///
/// This function constructs and returns the path to the Git index file within the specified Git
/// repository directory. The index file, also known as the staging area or cache, stores information
/// about the files and their changes to be committed.
///
/// # Arguments
///
/// * `git_dir`: A string representing the path to the Git repository directory.
///
/// # Returns
///
/// Returns a `String` containing the full path to the Git index file within the repository directory.
///
pub fn get_index_file_path(git_dir: &str) -> String {
    let mut index_file = PathBuf::from(git_dir);
    index_file.push("index");
    index_file.display().to_string()
}

/// Get the path to the Git ignore file (`.gitignore`) in a Git repository.
///
/// This function constructs and returns the path to the `.gitignore` file within the specified Git
/// repository directory. The `.gitignore` file contains patterns and rules for files and directories
/// that should be ignored by Git.
///
/// # Arguments
///
/// * `git_dir`: A string representing the path to the Git repository directory.
///
/// # Returns
///
/// Returns a `String` containing the full path to the `.gitignore` file within the repository directory.
///
pub fn get_git_ignore_path(git_dir: &str) -> String {
    let mut git_ignore_file = PathBuf::from(git_dir);
    git_ignore_file.push(".gitignore");
    git_ignore_file.display().to_string()
}

/// Get the current timestamp and offset for the local time zone.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether obtaining the timestamp was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * Unable to calculate the offset for the local time zone, resulting in an `Interrupted` error.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors.
pub fn get_timestamp() -> io::Result<(i64, String)> {
    let utc_now: DateTime<Utc> = Utc::now();

    let offset = match FixedOffset::west_opt(3 * 3600) {
        Some(off) => off,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Error getting timestamp.\n",
            ))
        }
    };

    let local_time = utc_now.with_timezone(&offset);

    let timestamp = local_time.timestamp();

    let offset_formatted_for_timestamp = format!(
        "{:+03}{:02}",
        offset.fix().local_minus_utc() / 3600,
        (offset.fix().local_minus_utc() % 3600) / 60
    );

    Ok((timestamp, offset_formatted_for_timestamp))
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use crate::commit;

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

    #[test]
    fn test_get_commit_1_parent() {
        if !fs::metadata("tests/utils/parents2").is_ok() {
            let _ = fs::create_dir_all("tests/utils/parents2");
        }

        let index_file = fs::File::create("tests/utils/parents2/index").unwrap();
        let mut index_file = io::BufWriter::new(index_file);
        //Write to the index file in the format hash path
        index_file
            .write_all(b"85628bead31d2c14e4a56113e524eab2ccff22c9\tREADME.md\n")
            .unwrap();

        fs::create_dir_all("tests/utils/parents2/refs/heads").unwrap();
        let mut main_file = fs::File::create("tests/utils/parents2/refs/heads/main").unwrap();
        main_file
            .write_all(b"0894f78e615131459e43d258070b5540081f1d82")
            .unwrap();

        let mut head_file = fs::File::create("tests/utils/parents2/HEAD").unwrap();
        head_file.write_all(b"ref: refs/heads/main").unwrap();

        let _ = fs::create_dir("tests/utils/parents2/objects");
        let result = commit::new_commit("tests/utils/parents2", "Mensaje", "").unwrap();

        let git_dir = "tests/utils/parents2";
        let mut expected_parents = Vec::new();
        expected_parents.push(result.clone());
        expected_parents.push("0894f78e615131459e43d258070b5540081f1d82".to_string());

        assert_eq!(
            get_branch_commit_history(&result, git_dir).unwrap(),
            expected_parents
        );
        let _ = fs::remove_dir_all("tests/utils/parents2");
    }

    #[test]
    fn test_get_commit_many_parents_set() {
        if !fs::metadata("tests/utils/parents3").is_ok() {
            let _ = fs::create_dir_all("tests/utils/parents3");
        }

        let git_dir_path = "tests/utils/parents3";

        let index_file = fs::File::create("tests/utils/parents3/index").unwrap();
        let mut index_file = io::BufWriter::new(index_file);
        //Write to the index file in the format hash path
        index_file
            .write_all(b"1f7a7a472abf3dd9643fd615f6da379c4acb3e3a\tREADME.md\n")
            .unwrap();

        fs::create_dir_all("tests/utils/parents3/refs/heads").unwrap();
        let mut main_file = fs::File::create("tests/utils/parents3/refs/heads/main").unwrap();
        main_file
            .write_all(b"a4a7dce85cf63874e984719f4fdd239f5145052e")
            .unwrap();

        let mut head_file = fs::File::create("tests/utils/parents3/HEAD").unwrap();
        head_file.write_all(b"ref: refs/heads/main").unwrap();

        let _ = fs::create_dir_all("tests/utils/parents3/objects").unwrap();
        let commit_1_hash = commit::new_commit(git_dir_path, "Mensaje", "").unwrap();
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let commit_2_hash = commit::new_commit(git_dir_path, "Aaaa", "").unwrap();
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\n3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/prueba/prueba3.c".as_bytes())
            .unwrap();
        let commit_3_hash = commit::new_commit(git_dir_path, "Holaaa", "").unwrap();

        let mut expected_parents = Vec::new();
        expected_parents.push(commit_3_hash.clone());
        expected_parents.push(commit_2_hash);
        expected_parents.push(commit_1_hash);
        expected_parents.push("a4a7dce85cf63874e984719f4fdd239f5145052e".to_string());

        assert_eq!(
            get_branch_commit_history(&commit_3_hash, git_dir_path).unwrap(),
            expected_parents
        );

        let _ = fs::remove_dir_all("tests/utils/parents3");
    }

    #[test]
    fn test_get_timestamp() -> io::Result<()> {
        let result = get_timestamp()?;

        let utc_now: DateTime<Utc> = Utc::now();

        let offset = FixedOffset::west_opt(3 * 3600).unwrap();
        let expected_local_time = utc_now.with_timezone(&offset);
        let expected_timestamp = expected_local_time.timestamp();

        let expected_offset_formatted = format!(
            "{:+03}{:02}",
            offset.fix().local_minus_utc() / 3600,
            (offset.fix().local_minus_utc() % 3600) / 60
        );

        assert_eq!(result.0, expected_timestamp);
        assert_eq!(result.1, expected_offset_formatted);
        Ok(())
    }
}
