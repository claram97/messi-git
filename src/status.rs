const BLOB: &str = "blob";

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::hash_object;
use crate::index::Index;
use crate::tree_handler::Tree;
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
///
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

/// Recursively find and write information about untracked files in a Git repository.
///
/// This function traverses the directory structure starting from the `current_directory`, compares
/// it with the files tracked in the Git `index`, and identifies files that are not tracked by Git.
/// It writes information about these untracked files to the provided `output`.
///
/// # Arguments
///
/// * `current_directory` - A reference to the current directory to start searching for untracked files.
/// * `base_directory` - A reference to the base directory of the Git repository.
/// * `index` - A reference to the Git index containing tracked file information.
/// * `output` - A mutable reference to a writer where the information about untracked files will be written.
///
/// # Errors
///
/// This function can return an `io::Result` which contains an `io::Error` if there is an issue
/// writing to the `output` or if there are issues with file operations.
///
pub fn find_untracked_files(
    current_directory: &Path,
    base_directory: &Path,
    index: &Index,
    output: &mut impl Write,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let entry_path = entry.path();

        if let Ok(relative_entry_path) = entry_path.strip_prefix(base_directory) {
            let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
            if !relative_entry_path_str.starts_with('.')
                && !index.path_should_be_ignored(&relative_entry_path_str)
                && !index.contains(&relative_entry_path_str)
            {
                if entry_path.is_dir() {
                    let buffer = format!("\x1b[31m\t\t{}x1b[0m\n", relative_entry_path_str);
                    output.write_all(buffer.as_bytes())?;
                    find_untracked_files(&entry_path, base_directory, index, output)?
                }
                if entry_path.is_file() {
                    let buffer = format!("\t\t{}\n", relative_entry_path_str);
                    output.write_all(buffer.as_bytes())?;
                }
            }
        } else {
            eprintln!("We've found some kind of mistake in git status");
        }
    }
    Ok(())
}


/// Find and write information about unstaged changes in a Git repository's index.
///
/// This function compares the hash of files in the provided `Index` with their current content
/// to identify modified files that haven't been staged for commit. It writes the information
/// about these changes to the provided `output`.
///
/// # Arguments
///
/// * `index` - A reference to the Git index containing file information.
/// * `git_dir` - A string representing the path to the Git repository directory.
/// * `output` - A mutable reference to a writer where the information about unstaged changes will be written.
///
/// # Errors
///
/// This function can return an `io::Result` which contains an `io::Error` if there is an issue
/// writing to the `output` or if there are issues with file operations, such as hashing the file content.
///
pub fn changes_to_be_committed(
    index: &Index,
    commit_tree: &Tree,
    output: &mut impl Write,
) -> io::Result<()> {
    for (path, hash) in index.iter() {
        if let Some(new_hash) = commit_tree.get_hash_from_path(path) {
            if hash.ne(&new_hash) {
                let buffer = format!("\x1b[31m\t\tmodified:\t {}\x1b[0m\n", path);
                output.write_all(buffer.as_bytes())?;
            }
        }
    }
    Ok(())
}

/// Find and write information about unstaged changes in a Git repository's index.
///
/// This function compares the hash of files in the provided `Index` with their current content
/// to identify modified files that haven't been staged for commit. It writes the information
/// about these changes to the provided `output`.
///
/// # Arguments
///
/// * `index` - A reference to the Git index containing file information.
/// * `git_dir` - A string representing the path to the Git repository directory.
/// * `output` - A mutable reference to a writer where the information about unstaged changes will be written.
///
/// # Errors
///
/// This function can return an `io::Result` which contains an `io::Error` if there is an issue
/// writing to the `output` or if there are issues with file operations, such as hashing the file content.
///
pub fn find_unstaged_changes(
    index: &Index,
    git_dir: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    for (path, hash) in index.iter() {
        let complete_path = git_dir.to_string() + "/" + path;
        println!("complete path: {}", complete_path);
        let new_hash = hash_object::hash_file_content(&complete_path, BLOB)?;
        if hash.ne(&new_hash) {
            let buffer = format!("\x1b[31m\t\tmodified:\t {}\x1b[0m\n", path);
            output.write_all(buffer.as_bytes())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{

        fs::{File, OpenOptions},
        io::Read,
    };

    use crate::{commit, init, tree_handler};

    use super::*;

    fn create_if_not_exists(path: &str, is_dir: bool) -> io::Result<()> {
        if !Path::new(path).exists() {
            if is_dir {
                std::fs::create_dir(path)?;
            } else {
                File::create(path)?;
            }
        }
        Ok(())
    }

    fn create_repo(path: &str) -> Result<(), io::Error> {
        create_if_not_exists(path, true)?;
        init::git_init(path, "current_branch", None)?;
        let gitignore_path = path.to_string() + "/.mgitignore";
        create_if_not_exists(&gitignore_path, false)?;
        let index_path = path.to_string() + "/.mgit/index";
        create_if_not_exists(&index_path, false)?;
        let file_1_path = path.to_string() + "/file1.txt";
        create_if_not_exists(&file_1_path, false)?;
        let file_2_path = path.to_string() + "/file2.txt";
        create_if_not_exists(&file_2_path, false)?;
        let file_3_path = path.to_string() + "/.mgit/file3.txt";
        create_if_not_exists(&file_3_path, false)?;
        let dir = path.to_string() + "/dir";
        create_if_not_exists(&dir, true)?;
        let file = path.to_string() + "/dir/file";
        create_if_not_exists(&file, false)?;
        let index_content = "032356 file1.txt\n01211 file2.txt\n01010101 .mgit/file3.txt\n";
        let gitignore_content = "dir\n";
        let mut index_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(index_path)?;
        index_file.write_all(index_content.as_bytes())?;
        let mut gitignore_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(gitignore_path)?;
        gitignore_file.write_all(gitignore_content.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_no_changes_to_be_committed_throws_back_no_output() -> Result<(), io::Error> {
        create_repo("tests/status_fake_repo")?;
        let git_dir: &str = "tests/status_fake_repo";
        let mut output: Vec<u8> = vec![];
        changes_to_be_committed_for_test(&git_dir, true, &mut output)?;
        assert!(output.is_empty());
        std::fs::remove_dir_all("tests/status_fake_repo")?;
        Ok(())
    }

    #[test]
    fn test_changes_to_be_committed_throws_back_output_files_since_theres_been_a_commit(
    ) -> Result<(), io::Error> {
        create_repo("tests/status_feik_repo")?;
        let mut output: Vec<u8> = vec![];
        changes_to_be_committed_for_test("tests/status_feik_repo", true, &mut output)?;
        let index_file_path = "tests/status_feik_repo/.mgit/index";
        let new_index_content = "032357 file1.txt\n01211 file2.txt\n01010101 .mgit/file3.txt\n";
        let mut new_index_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(index_file_path)?;
        new_index_file.write_all(new_index_content.as_bytes())?;
        let mut new_output: Vec<u8> = vec![];
        changes_to_be_committed_for_test("tests/status_feik_repo", false, &mut new_output)?;

        let result: Result<String, std::string::FromUtf8Error> = String::from_utf8(new_output);
        if result.is_ok() {
            let resultado = result.unwrap();
            assert!(resultado.contains("modified:"));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Parent hash not found",
            ));
        }
        std::fs::remove_dir_all("tests/status_feik_repo")?;
        Ok(())
    }

    fn changes_to_be_committed_for_test(
        path: &str,
        commit: bool,
        mut output: &mut impl Write,
    ) -> Result<(), io::Error> {
        let git_dir = path.to_string();
        let mgit_path = path.to_string() + "/.mgit";
        let git_ignore_path = path.to_string() + "/.mgitignore";
        let index_file_path = path.to_string() + "/.mgit/index";
        let index = Index::load(&index_file_path, &git_dir, &git_ignore_path)?;
        if commit == true {
            let _commit_result = commit::new_commit(&mgit_path, "message", &git_ignore_path);
        }
        let current_branch = path.to_string() + "/.mgit/refs/heads/current_branch";
        let mut current_commit_file = File::open(&current_branch)?;
        let mut commit_hash = String::new();
        current_commit_file.read_to_string(&mut commit_hash)?;
        let tree = tree_handler::load_tree_from_commit(&commit_hash, &mgit_path)?;
        changes_to_be_committed(&index, &tree, &mut output)?;
        Ok(())
    }

    #[test]
    fn test_unstaged_changes_detected_when_changes_not_added() -> Result<(), io::Error> {
        create_repo("tests/status_repo")?;
        let working_dir = "tests/status_repo";
        let git_dir = "tests/status_repo/.mgit";
        let git_ignore_path = "tests/status_repo/.mgitignore";
        let index_file_path = "tests/status_repo/.mgit/index";
        let index_content = "032356 file1.txt\n01211 file2.txt\n01010101 .mgit/file3.txt\n";
        let gitignore_content = "dir\n";
        let mut index_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(index_file_path)?;
        index_file.write_all(index_content.as_bytes())?;
        let mut gitignore_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(git_ignore_path)?;
        gitignore_file.write_all(gitignore_content.as_bytes())?;
        let index = Index::load(&index_file_path, &git_dir, &git_ignore_path)?;
        let mut output: Vec<u8> = vec![];
        find_unstaged_changes(&index, &working_dir, &mut output)?;
        assert!(!output.is_empty());
        std::fs::remove_dir_all("tests/status_repo")?;
        Ok(())
    }

    #[test]
    fn test_changes_not_added_with_uncommitted_changes() -> Result<(), io::Error> {
        create_repo("tests/status_repo_2")?;
        let git_dir = "tests/status_repo_2";
        let git_ignore_path = "tests/status_repo_2/.mgitignore";
        let index_file_path = "tests/status_repo_2/.mgit/index";
        let gitignore_content = "dir\n";
        let mut gitignore_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(git_ignore_path)?;
        gitignore_file.write_all(gitignore_content.as_bytes())?;
        let mut file1 = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("tests/status_repo_2/file1.txt")?;
        let file1_content = "soy file1";
        file1.write_all(file1_content.as_bytes())?;
        let hash_file1 = hash_object::hash_file_content("tests/status_repo_2/file1.txt", "blob")?;
        let mut file2 = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("tests/status_repo_2/file2.txt")?;
        let file2_content = "soy file2";
        file2.write_all(file2_content.as_bytes())?;
        let hash_file2 = hash_object::hash_file_content("tests/status_repo_2/file2.txt", "blob")?;
        let mut file3 = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("tests/status_repo_2/.mgit/file3.txt")?;
        let file3_content = "soy file3";
        file3.write_all(file3_content.as_bytes())?;
        let hash_file3 =
            hash_object::hash_file_content("tests/status_repo_2/.mgit/file3.txt", "blob")?;
        let index_content = hash_file1
            + " file1.txt\n"
            + &hash_file2
            + " file2.txt\n"
            + &hash_file3
            + " .mgit/file3.txt\n";
        let mut index_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(index_file_path)?;
        index_file.write_all(index_content.as_bytes())?;
        let index = Index::load(&index_file_path, &git_dir, &git_ignore_path)?;
        let mut output: Vec<u8> = vec![];
        find_unstaged_changes(&index, &git_dir, &mut output)?;
        assert!(output.is_empty());
        std::fs::remove_dir_all("tests/status_repo_2")?;
        Ok(())
    }

    #[test]
    fn test_find_untracked_files() -> Result<(), io::Error> {
        create_repo("tests/status_repo_")?;
        let git_dir = Path::new("tests/status_repo_");
        let git_ignore_path = "tests/status_repo_/.mgitignore";
        let index_file_path = "tests/status_repo_/.mgit/index";
        let mut output: Vec<u8> = vec![];
        let index = Index::load(
            &index_file_path,
            &git_dir.to_string_lossy().to_string(),
            &git_ignore_path,
        )?;

        create_if_not_exists("tests/status_repo_/file4.txt", false)?;
        find_untracked_files(&git_dir, &git_dir, &index, &mut output)?;
        assert!(!output.is_empty());
        let resultado = String::from_utf8(output);
        if resultado.is_ok() {
            let string = resultado.unwrap();
            assert!(string.contains("file4.txt"));
        } else {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error.\n"));
        }
        std::fs::remove_dir_all("tests/status_repo_")?;

        Ok(())
    }
}
