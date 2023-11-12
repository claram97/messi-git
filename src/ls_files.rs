use std::{io::{self, Write}, fs, path::Path};

use crate::{index::Index, hash_object};
const BLOB: &str = "blob";

/// Recursively lists files in the specified directory that are present in the given index.
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
/// Returns an `io::Result` indicating success or failure. If an error occurs during directory traversal or writing to the output, an `io::Error` is returned.
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
fn list_files_in_index(working_dir : &str, git_dir: &str, current_directory : &str, line: Vec<String>, index : &Index, output: &mut impl Write) -> io::Result<()> {
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_path_str = entry_path.to_string_lossy().to_string();
        if let Ok(relative_entry_path) = entry_path.strip_prefix(working_dir) {
            let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
            if index.contains(&relative_entry_path_str)
            {
                if entry_path.is_dir() {
                    let buffer = format!("{}\n", relative_entry_path_str);
                    output.write_all(buffer.as_bytes())?;
                    let cloned_line = line.clone();
                    list_files_in_index(working_dir, git_dir, &entry_path_str, cloned_line, index, output)?;
                }
                if entry_path.is_file() {
                    let buffer = format!("{}\n", relative_entry_path_str);
                    output.write_all(buffer.as_bytes())?;
                }
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
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
fn list_untracked_files(working_dir : &str, git_dir: &str, current_directory : &str, line: Vec<String>, index : &Index, output: &mut impl Write) -> io::Result<()> {
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_path_str = entry_path.to_string_lossy().to_string();
        if let Ok(relative_entry_path) = entry_path.strip_prefix(working_dir) {
            let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
            if !index.path_should_be_ignored(&relative_entry_path_str)
                && !index.contains(&relative_entry_path_str)
            {
                if entry_path.is_dir() {
                    let cloned_line = line.clone();
                    git_ls_files(working_dir, git_dir, &entry_path_str, cloned_line, index, output)?;
                }
                if entry_path.is_file() {
                    let buffer = format!("{}\n", relative_entry_path_str);
                    output.write_all(buffer.as_bytes())?;
                }
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
fn list_modified_files(working_dir : &str, index : &Index, output: &mut impl Write) -> io::Result<()> {
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
//let current_directory = std::env::current_dir()?;

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
pub fn git_ls_files(working_dir : &str, git_dir: &str, current_directory : &str, line: Vec<String>, index : &Index, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 2 || (line.len() == 3 && line[1].eq("-c")) {
        list_files_in_index(working_dir, git_dir, current_directory, line, index, output)?;
    }
    else if line.len() == 3 {
        if line[2].eq("-o") {
            list_untracked_files(working_dir, git_dir, current_directory, line, index, output)?;
        }
        else if line[2].eq("-m") {
            list_modified_files(working_dir, index, output)?;
        }
        else {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error.\n"));
        }
    }
    Ok(())
}