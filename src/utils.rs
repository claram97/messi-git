use std::{collections::HashSet, io, path::PathBuf};

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

pub fn get_branch_commit_history_set(commit_hash: &str, git_dir: &str) -> io::Result<HashSet<String>> {
    let mut parents = HashSet::new();
    parents.insert(commit_hash.to_string());
    let mut commit_parent = commit::get_parent_hash(commit_hash, git_dir);
    while let Ok(parent) = commit_parent {
        parents.insert(parent.clone());
        commit_parent = commit::get_parent_hash(&parent, git_dir);
    }
    Ok(parents)
}

pub fn get_index_file_path(git_dir: &str) -> String {
    let mut index_file = PathBuf::from(git_dir);
    index_file.push("index");
    index_file.display().to_string()
}

pub fn get_git_ignore_path(git_dir: &str) -> String {
    let mut git_ignore_file = PathBuf::from(git_dir);
    git_ignore_file.push(".gitignore");
    git_ignore_file.display().to_string()
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
            .write_all(b"1f7a7a472abf3dd9643fd615f6da379c4acb3e3a\tREADME.md\n")
            .unwrap();

        fs::create_dir_all("tests/utils/parents2/refs/heads").unwrap();
        let mut main_file = fs::File::create("tests/utils/parents2/refs/heads/main").unwrap();
        main_file
            .write_all(b"a4a7dce85cf63874e984719f4fdd239f5145052e")
            .unwrap();

        let mut head_file = fs::File::create("tests/utils/parents2/HEAD").unwrap();
        head_file.write_all(b"ref: refs/heads/main").unwrap();

        let _ = fs::create_dir("tests/utils/parents2/objects").unwrap();
        let result = commit::new_commit("tests/utils/parents2", "Mensaje", "").unwrap();

        let git_dir = "tests/utils/parents2";
        let mut expected_parents = Vec::new();
        expected_parents.push(result.clone());
        expected_parents.push("a4a7dce85cf63874e984719f4fdd239f5145052e".to_string());

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
            .write_all("\nhashhashhashhash5 src/prueba/prueba2.c".as_bytes())
            .unwrap();
        let commit_2_hash = commit::new_commit(git_dir_path, "Aaaa", "").unwrap();
        let mut index_file = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir_path.to_string() + "/index")
            .unwrap();
        index_file
            .write_all("\nhashhashhashhash6 src/prueba/prueba3.c".as_bytes())
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
    }
}
