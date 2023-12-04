use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::logger::Logger;
use crate::utils::get_current_time;
use crate::{hash_object, index::Index};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
const BLOB: &str = "blob";

/// Lists files present in the provided index.
///
/// # Arguments
///
/// * `index` - The index containing the tracked files.
/// * `output` - A mutable reference to an implementor of the `Write` trait for outputting the file list.
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. If an error occurs during writing to the output, an `io::Error` is returned.
///
/// # Examples
///
/// Basic usage:
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
fn list_files_in_index(index: &Index, output: &mut impl Write) -> io::Result<()> {
    for (path, _hash) in index.iter() {
        let path_to_print = format!("{}\n", path);
        output.write_all(path_to_print.as_bytes())?;
    }
    Ok(())
}

// Process untracked files and directories.
///
/// This function processes untracked items, handling both directories and files. For directories,
/// it recursively lists the files using the `git_ls_files` function. For files, it writes the
/// relative entry path to the specified output.
///
/// # Arguments
///
/// - `relative_entry_path_str`: The relative path of the untracked entry as a string.
/// - `entry_path`: The path to the untracked entry.
/// - `line`: A vector of strings representing additional information about the entry.
/// - `working_dir`: The path to the working directory.
/// - `git_dir`: The path to the Git directory.
/// - `index`: A reference to the Git index.
/// - `output`: A mutable reference to an implementation of the `Write` trait where the output
///   will be written.
///
/// # Returns
///
/// Returns an `io::Result<()>`, indicating success or an `io::Error` if any I/O operation fails.
///
fn process_untracked(
    relative_entry_path_str: &str,
    entry_path: &Path,
    line: Vec<String>,
    working_dir: &str,
    git_dir: &str,
    index: &Index,
    output: &mut impl Write,
) -> io::Result<()> {
    let entry_path_str = entry_path.to_string_lossy().to_string();

    if entry_path.is_dir() {
        git_ls_files(working_dir, git_dir, &entry_path_str, line, index, output)?;
    }
    if entry_path.is_file() {
        let buffer = format!("{}\n", relative_entry_path_str);
        output.write_all(buffer.as_bytes())?;
    }
    Ok(())
}

/// Recursively lists untracked files in the specified directory by comparing with the given index.
///
/// # Arguments
///
/// * `working_dir` - The root directory of the working tree.
/// * `git_dir` - The path to the `.git` directory.
/// * `current_directory` - The current directory to list untracked files from.
/// * `line` - The original command line arguments.
/// * `index` - The index containing the tracked files.
/// * `output` - A mutable reference to an implementor of the `Write` trait for outputting the file list.
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. If an error occurs during directory traversal or writing to the output, an `io::Error` is returned.
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
fn list_untracked_files(
    working_dir: &str,
    git_dir: &str,
    current_directory: &str,
    line: Vec<String>,
    index: &Index,
    output: &mut impl Write,
) -> io::Result<()> {
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let entry_path = entry.path();
        if let Ok(relative_entry_path) = entry_path.strip_prefix(working_dir) {
            let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
            if !index.path_should_be_ignored(&relative_entry_path_str)
                && !index.contains(&relative_entry_path_str)
            {
                let cloned_line = line.clone();
                process_untracked(
                    &relative_entry_path_str,
                    &entry_path,
                    cloned_line,
                    working_dir,
                    git_dir,
                    index,
                    output,
                )?;
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error.\n"));
        }
    }
    Ok(())
}

/// Recursively lists untracked files in the specified directory by comparing with the given index.
///
/// # Arguments
///
/// * `working_dir` - The root directory of the working tree.
/// * `git_dir` - The path to the `.git` directory.
/// * `current_directory` - The current directory to list untracked files from.
/// * `line` - The original command line arguments.
/// * `index` - The index containing the tracked files.
/// * `output` - A mutable reference to an implementor of the `Write` trait for outputting the file list.
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. If an error occurs during directory traversal or writing to the output, an `io::Error` is returned.
///
/// # Examples
///
/// Basic usage:
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
fn list_modified_files(
    working_dir: &str,
    index: &Index,
    output: &mut impl Write,
) -> io::Result<()> {
    for (path, hash) in index.iter() {
        let complete_path_string = working_dir.to_string() + "/" + path;
        let complete_path = Path::new(&complete_path_string);
        if complete_path.is_file() {
            let new_hash = hash_object::hash_file_content(&complete_path_string, BLOB)?;
            if hash.ne(&new_hash) {
                let buffer = format!("{}\n", path);
                output.write_all(buffer.as_bytes())?;
            }
        }
    }
    Ok(())
}

/// Logs the 'git ls-files' command with the specified working directory, Git directory, current directory, and line.
///
/// This function logs the 'git ls-files' command with the provided working directory, Git directory,
/// current directory, and command line arguments to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `working_dir` - A string representing the root directory of the working tree.
/// * `git_dir` - A string representing the path to the `.git` directory.
/// * `current_directory` - A string representing the current directory to list files from.
/// * `line` - A vector of strings representing the original command line arguments.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_ls_files(
    working_dir: &str,
    git_dir: &str,
    current_directory: &str,
    line: &[String],
) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git ls-files': Working Directory '{}', Git Directory '{}', Current Directory '{}', Line '{:?}', {}",
        working_dir,
        git_dir,
        current_directory,
        line,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Lists files based on the provided command line arguments in a manner similar to the 'git ls-files' command.
///
/// # Arguments
///
/// * `working_dir` - The root directory of the working tree.
/// * `git_dir` - The path to the `.git` directory.
/// * `current_directory` - The current directory to list files from.
/// * `line` - The original command line arguments.
/// * `index` - The index containing the tracked files.
/// * `output` - A mutable reference to an implementor of the `Write` trait for outputting the file list.
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. If an error occurs during file listing or writing to the output, an `io::Error` is returned.
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
pub fn git_ls_files(
    working_dir: &str,
    git_dir: &str,
    current_directory: &str,
    line: Vec<String>,
    index: &Index,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() == 2 || (line.len() == 3 && line[2].eq("-c")) {
        list_files_in_index(index, output)?;
    } else if line.len() == 3 {
        if line[2].eq("-o") {
            list_untracked_files(
                working_dir,
                git_dir,
                current_directory,
                line.clone(),
                index,
                output,
            )?;
        } else if line[2].eq("-m") {
            list_modified_files(working_dir, index, output)?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error.\n"));
        }
    }
    log_ls_files(working_dir, git_dir, current_directory, &line)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{File, OpenOptions};

    use crate::{configuration::GIT_DIR_FOR_TEST, init};

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
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
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
        let index_content = "2c0611919ae5d4d765fc49cef961d67886411cad file1.txt\n0bad9566be86bcf4493d69b6b55d73137efd45a1 file2.txt\n34ae409f501db061fbf67c43085e7a06a9537359 .mgit/file3.txt\n";
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
    fn ls_files_with_invalid_option_returns_error() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_1";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string(), "-w".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files("", "", "", line, &index, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn ls_files_lists_files_on_index() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_2";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(repo_path, &git_dir, repo_path, line, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("file1.txt"));
        assert!(string.contains("file2.txt"));
        assert!(string.contains(".mgit/file3.txt"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn ls_files_c_option_lists_files_on_index() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_3";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string(), "-c".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(repo_path, &git_dir, repo_path, line, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("file1.txt"));
        assert!(string.contains("file2.txt"));
        assert!(string.contains(".mgit/file3.txt"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn ls_files_other_option_lists_untracked_files() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_4";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string(), "-o".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(repo_path, &git_dir, repo_path, line, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains(".mgitignore"));
        assert!(string.contains(".mgit"));
        assert!(string.contains(".mgit/config"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn ls_files_modified_option_lists_modified_files() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_5";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string(), "-m".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(repo_path, &git_dir, repo_path, line, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("file1.txt"));
        assert!(string.contains("file2.txt"));
        assert!(string.contains(".mgit/file3.txt"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn list_works_correctly() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_6";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = list_files_in_index(&index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("file1.txt"));
        assert!(string.contains("file2.txt"));
        assert!(string.contains(".mgit/file3.txt"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn list_untracked_works_correctly() -> io::Result<()> {
        let repo_path = "tests/ls_files_repo_7";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let line: Vec<String> = vec!["git".to_string(), "ls-files".to_string(), "-o".to_string()];
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result =
            list_untracked_files(repo_path, &git_dir, repo_path, line, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains(".mgitignore"));
        assert!(string.contains(".mgit"));
        assert!(string.contains(".mgit/config"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }

    #[test]
    fn list_modified_works_correctly() -> io::Result<()> {
        let repo_path = "tests/ls_files_8";
        create_repo(repo_path)?;
        let index_path = format!("{}/{}", repo_path, ".mgit/index");
        let gitignore_path = format!("{}/{}", repo_path, ".mgitignore");
        let git_dir = format!("{}/{}", repo_path, ".mgit");
        let index = Index::load(&index_path, &git_dir, &gitignore_path)?;
        let mut output: Vec<u8> = vec![];
        let result = list_modified_files(repo_path, &index, &mut output);
        assert!(result.is_ok());
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("file1.txt"));
        assert!(string.contains("file2.txt"));
        assert!(string.contains(".mgit/file3.txt"));
        std::fs::remove_dir_all(repo_path)?;
        Ok(())
    }
}
