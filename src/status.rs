pub(crate) const NAME_OF_INDEX_FILE: &str = "index-file";
pub(crate) const NAME_OF_GIT_DIRECTORY: &str = ".git";
use messi::hash_object;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::{fs, io::ErrorKind};
/**
 * Recursively searches for the Git directory starting from the current directory and moving upwards.
 *
 * # Example
 *
 * ```
 * use std::env;
 *
 * let git_directory = find_git_directory();
 * match git_directory {
 *     Some(dir) => println!("Git directory found at: {}", dir),
 *     None => println!("No Git directory found in the path."),
 * }
 * ```
 *
 * The function returns `Some(path)` if a valid Git directory is found, where `path` is the path to the found Git directory.
 * If no Git directory is found anywhere in the path upwards from the current directory, the function returns `None`.
 */
pub fn find_git_directory() -> Option<String> {
    if let Ok(current_dir) = env::current_dir() {
        let mut current_dir = current_dir;

        loop {
            let git_dir = current_dir.join(NAME_OF_GIT_DIRECTORY);
            if git_dir.exists() && git_dir.is_dir() {
                return Some(git_dir.display().to_string());
            }

            if !current_dir.pop() {
                break;
            }
        }
    }

    None
}

/**
 * Reads the Git index file to check for changes in tracked files.
 *
 * # Arguments
 *
 * - `file`: A mutable reference to a `File` representing the Git index file.
 *
 * # Returns
 *
 * Returns a result containing `Ok(())` if the operation is successful, indicating that the index file has been processed.
 * If there are any errors encountered during file reading or hash computation, an `io::Result` containing an error is returned.
 *
 * # Example
 *
 * ```
 * use std::fs::File;
 * use std::io;
 *
 * let mut index_file = File::open("index")?;
 * match read_index_file(&mut index_file) {
 *     Ok(()) => println!("Index file successfully processed."),
 *     Err(e) => eprintln!("Error while processing index file: {}", e),
 * }
 * ```
 */
fn read_index_file(file: &mut File) -> io::Result<()> {
    let git_directory = match find_git_directory() {
        Some(dir) => dir,
        None => return Err(io::Error::new(ErrorKind::NotFound, "Git directory not found."))
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                let splitted_line: Vec<&str> = line_content.split_whitespace().collect();
                if let Some(parent) = Path::new(&git_directory).parent() {
                    let file_path = parent.to_string_lossy().to_string() + "/" + splitted_line[1];
                    let hash = hash_object::hash_file_content(&file_path)?;
                    if !hash.eq(splitted_line[0]) {
                        println!("File {} has changed since last commit.", splitted_line[1]);
                    }
                }   
            }
            Err(e) => {
                eprintln!("Error trying to read the line: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

/**
 * Finds and checks files that have changed since the last Git commit in the current repository.
 *
 * This function searches for the Git directory in the current repository, then reads the Git index file
 * to compare the files' hash values with their last known state. It reports any files that have changed
 * since the last commit.
 *
 * # Returns
 *
 * Returns an `io::Result` where `Ok(())` indicates that the operation is successful, and any errors
 * encountered during file access and processing are returned as an error result.
 *
 * # Example
 *
 * ```
 * use std::io;
 *
 * match find_files_that_changed_since_last_commit() {
 *     Ok(()) => println!("No changes detected since the last commit."),
 *     Err(e) => eprintln!("Error while checking for changes: {}", e),
 * }
 * ```
 */
pub fn find_files_that_changed_since_last_commit() -> io::Result<()> {
    match find_git_directory() {
        Some(dir) => {
            let file_path = dir + "/" + NAME_OF_INDEX_FILE;
            let mut file = File::open(file_path)?;
            read_index_file(&mut file)?;
            Ok(())
        }
        None => Err(io::Error::new(
            ErrorKind::NotFound,
            "Git index file couldn't be opened.",
        )),
    }
}

/**
 * Recursively searches for files and directories in the current directory and its subdirectories.
 *
 * This function traverses the file system starting from the `current_directory` and searches for
 * files and directories. It compares the found entries against a list of files to be matched
 * and a list of directories to be ignored. Any entries not matching the list are printed to the console.
 *
 * # Arguments
 *
 * - `current_directory`: A reference to the `Path` representing the current directory to start the search.
 * - `files_list`: A `HashSet` containing filenames to be matched.
 * - `ignored_folders`: A `HashSet` containing folder names to be ignored.
 *
 * # Returns
 *
 * Returns `Ok(())` if the operation is successful, and any errors encountered during file system access
 * are returned as an error result in the `io::Result`.
 *
 * # Example
 *
 * ```
 * use std::io;
 * use std::path::Path;
 * use std::collections::HashSet;
 *
 * let current_dir = Path::new("/path/to/start/search");
 * let files_to_match: HashSet<String> = ["file1.txt", "file2.txt"].iter().cloned().collect();
 * let ignored_folders: HashSet<String> = ["target", "node_modules"].iter().cloned().collect();
 *
 * match search_in_directory(&current_dir, &files_to_match, &ignored_folders) {
 *     Ok(()) => println!("Search completed successfully."),
 *     Err(e) => eprintln!("Error during search: {}", e),
 * }
 * ```
 */
fn search_in_directory(
    current_directory: &Path,
    files_list: &HashSet<String>,
    ignored_folders: &HashSet<String>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_path_str = entry_path.to_string_lossy().to_string();

        if ignored_folders
            .iter()
            .any(|ignored| entry_path_str.starts_with(ignored))
        {
            continue;
        }

        if !files_list.contains(&entry_path_str)
            && entry_path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(true, |s| !s.starts_with('.'))
        {
            println!("Not found in the list: {:?}", entry_path);
        }

        if entry_path.is_dir()
            && entry_path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| !s.starts_with('.'))
        {
            search_in_directory(&entry_path, files_list, ignored_folders)?;
        }
    }
    Ok(())
}

/**
 * Recursively searches for files not found in the specified list while considering ignored folders.
 *
 * This function reads a list of files to be matched, a list of folders to be ignored, and a base directory.
 * It then recursively searches the file system starting from the `base_directory` and compares the found entries
 * against the list of files to be matched and the list of folders to be ignored. Any entries not matching the list
 * are reported to the console.
 *
 * # Arguments
 *
 * - `base_directory`: A reference to the `Path` representing the base directory to start the search.
 * - `file_list_path`: A string specifying the path to the file containing the list of files to be matched.
 * - `ignore_list_path`: A string specifying the path to the file containing the list of folders to be ignored.
 *
 * # Returns
 *
 * Returns `Ok(())` if the operation is successful, and any errors encountered during file system access
 * are returned as an error result in the `io::Result`.
 *
 * # Example
 *
 * ```
 * use std::io;
 * use std::path::Path;
 *
 * let base_dir = Path::new("/path/to/start/search");
 * let file_list_path = "file_list.txt";
 * let ignore_list_path = "ignore_list.txt";
 *
 * match find_not_in_list(&base_dir, file_list_path, ignore_list_path) {
 *     Ok(()) => println!("Search completed successfully."),
 *     Err(e) => eprintln!("Error during search: {}", e),
 * }
 * ```
 */
pub fn find_missing_files_in_directory(
    base_directory: &Path,
    file_list_path: &str,
    ignore_list_path: &str,
) -> Result<(), io::Error> {
    let mut files_list = HashSet::new();
    let mut ignored_folders = HashSet::new();

    let list_reader = io::BufReader::new(fs::File::open(file_list_path)?);
    for line in list_reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            files_list.insert(parts[1].to_string());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Something went wrong with index-file format.",
            ));
        }
    }

    let ignore_reader = io::BufReader::new(fs::File::open(ignore_list_path)?);
    for line in ignore_reader.lines() {
        let line = line?;
        let complete_path = base_directory.to_string_lossy().to_string() + &line;
        ignored_folders.insert(complete_path);
    }

    search_in_directory(base_directory, &files_list, &ignored_folders)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, remove_dir_all},
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    /// Creates a temporary directory with a Git repository for testing.
    ///
    /// This function sets up a temporary directory and simulates a Git repository
    /// within it. It creates a `.git` directory and a fake `HEAD` file, and sets
    /// the `GIT_DIR` environment variable to point to the test repository.
    ///
    /// # Panics
    ///
    /// This function panics if it fails to create the necessary directories or files.
    ///
    fn create_test_git_repository() {
        let temp_dir = env::temp_dir().join("test_git_repository");
        create_dir(&temp_dir).expect("Failed to create a temporary directory");
        let repo_dir = temp_dir.join(NAME_OF_GIT_DIRECTORY);

        create_dir(&repo_dir).expect("Failed to create .git directory");

        let head_file = repo_dir.join("HEAD");
        let mut file = File::create(&head_file).expect("Failed to create HEAD file");
        file.write_all(b"ref: refs/heads/master\n")
            .expect("Failed to write to HEAD file");

        env::set_var("GIT_DIR", &repo_dir);
        env::set_current_dir(temp_dir).expect("Failed to set the current directory");
    }

    /// Cleans up the test Git repository created for testing.
    ///
    /// This function removes the temporary directory created for testing, which
    /// also cleans up the test Git repository. It resets the `GIT_DIR` environment
    /// variable and ensures that no leftover files or directories exist.
    ///
    /// # Panics
    ///
    /// This function panics if it fails to remove the temporary directory.
    ///
    fn cleanup_test_git_repository() {
        env::remove_var("GIT_DIR");
        let temp_dir = env::temp_dir().join("test_git_repository");
        remove_dir_all(temp_dir).expect("Failed to remove the temporary directory");
    }

    #[test]
    fn test_try_to_find_git_directory_that_exists() {
        // Create a test Git repository
        create_test_git_repository();

        // Call the function being tested
        let result = super::find_git_directory();

        // Assert that the result is Some and the directory exists
        assert!(result.is_some());

        // Clean up the test Git repository
        cleanup_test_git_repository();
    }

    /// Creates a unique temporary directory for testing without a Git repository.
    ///
    /// This function generates a unique directory name based on the current timestamp
    /// and creates a temporary directory for testing where there is no Git repository.
    /// It simulates the absence of a Git repository.
    ///
    /// # Panics
    ///
    /// This function panics if it fails to create the necessary directory or set
    /// the current directory.
    ///
    fn create_test_directory_without_git() {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let unique_name = format!("test_directory_without_git_{}", since_epoch.as_micros());

        let temp_dir = env::temp_dir().join(&unique_name);

        create_dir(&temp_dir).expect("Failed to create a temporary directory");

        env::set_current_dir(&temp_dir).expect("Failed to set the current directory");
    }

    /// Cleans up the test directory without a Git repository created for testing.
    ///
    /// This function removes the temporary directory created for testing and ensures
    /// that there is no Git directory in the temporary directory. It checks that the
    /// temporary directory exists, is a directory, and that the `.git` directory
    /// does not exist.
    ///
    /// # Panics
    ///
    /// This function panics if it fails to remove the temporary directory or if any
    /// unexpected conditions are encountered during cleanup.
    ///
    fn cleanup_test_directory_without_git() {
        let temp_dir = env::temp_dir().join("test_directory_without_git");
        assert!(temp_dir.exists(), "Temporary directory should exist");
        assert!(
            temp_dir.is_dir(),
            "Temporary directory should be a directory"
        );
        assert!(
            !temp_dir.join(NAME_OF_GIT_DIRECTORY).exists(),
            "Git directory should not exist in the temporary directory"
        );
    }

    #[test]
    fn test_try_to_find_git_directory_that_does_not_exists() {
        create_test_directory_without_git();
        let result = super::find_git_directory();
        assert!(result.is_none(), "Git directory should not exist");
        cleanup_test_directory_without_git();
    }
}
