use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};
use std::path::{Path};

/// Removes a file from the file system.
///
/// This function attempts to remove a file from the file system. If the file does not exist, it will
/// display an error message and return an `io::Result` indicating the error. If the file is
/// successfully removed, the function returns `Ok(())`.
///
/// # Arguments
///
/// * `file` - A string representing the path to the file to be removed.
///
/// # Errors
///
/// This function returns an `io::Result`, which can contain an `io::Error` in case of any errors
/// during the file removal process.
///
fn remove_file(file: &str) -> io::Result<()> {
    if !Path::new(file).exists() {
        eprintln!("The file '{}' does not exist.", file);
    }

    fs::remove_file(file)?;
    Ok(())
}

/// Removes a file from the Git index.
///
/// This function attempts to remove a file from the Git index, simulating the process of
/// removing an entry from the index file (".git/index"). If the file is successfully removed
/// from the index, the function returns `Ok(())`. If the file is not found in the index,
/// it returns an error of kind `io::ErrorKind::NotFound`.
///
/// # Arguments
///
/// * `file` - A string representing the path to the file to be removed from the index.
///
/// # Errors
///
/// This function returns an `io::Result`, which can contain an `io::Error` with the kind
/// `io::ErrorKind::NotFound` if the file is not found in the index. Other I/O errors may occur
/// during the removal process.
///
pub fn remove_file_from_index(file: &str) -> io::Result<()> {
    // Open the Git index file (".git/index")
    let index_path = Path::new(".git").join("index");
    let mut index_file = File::open(&index_path)?;

    // Read the entire index file into memory
    let mut index_contents = Vec::new();
    index_file.read_to_end(&mut index_contents)?;

    // Find and remove the entry corresponding to the specified file
    let mut offset = 12; // Skip the header
    let mut found = false;

    while offset < index_contents.len() {
        let entry_size = read_u32(&index_contents[offset..offset + 4]) as usize;
        let entry_path = String::from_utf8_lossy(&index_contents[offset + 62..])
            .trim_matches(char::from(0))
            .to_string();

        if entry_path == file {
            // Skip this entry to effectively remove it
            index_contents.drain(offset..offset + entry_size);
            found = true;
            break;
        }

        offset += entry_size;
    }

    if !found {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found in the index: {}", file),
        ));
    }

    // Write the modified index contents back to the index file
    let mut new_index_file = File::create(&index_path)?;
    new_index_file.write_all(&index_contents)?;

    Ok(())
}

/// Reads four bytes from a slice and converts them into a 32-bit unsigned integer (u32).
///
/// This function takes a slice of four bytes and interprets them as a big-endian 32-bit
/// unsigned integer (u32). It performs the necessary bit shifting and bitwise OR operations
/// to convert the bytes into a u32 value.
///
/// # Arguments
///
/// * `bytes` - A slice containing exactly four bytes that represent the u32 value.
///
/// # Returns
///
/// Returns the 32-bit unsigned integer (u32) value obtained by interpreting the input bytes.
///
fn read_u32(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

/// Removes a file or directory from the file system.
///
/// This function takes a path to a file or directory and attempts to remove it from the file system.
/// If the specified file or directory exists, it will be removed. If it doesn't exist, an error message
/// will be displayed in the standard error stream.
///
/// # Arguments
///
/// * `file` - A string representing the path to the file or directory to be removed.
///
/// # Returns
///
/// Returns a Result indicating whether the operation was successful or if an error occurred.
///
/// # Errors
///
/// If the specified file or directory does not exist or if an error occurs during the removal process,
/// an error is returned. The error message will indicate the cause of the failure.
///
fn remove_file_or_directory(file: &str) -> io::Result<()> {
    if !Path::new(file).exists() {
        eprintln!("The file or directory '{}' does not exist.", file);
    }

    if Path::new(file).is_dir() {
        fs::remove_dir_all(file)?;
    } else {
        fs::remove_file(file)?;
    }
    Ok(())
}

/// Parses command-line arguments and checks for the minimum required arguments.
///
/// This function takes a slice of command-line arguments and checks whether there are enough
/// arguments for the operation to proceed. If the minimum required arguments are not provided,
/// an error message is returned.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
///
/// # Returns
///
/// Returns a Result indicating whether the operation was successful or if an error occurred.
///
/// # Errors
///
/// If the minimum required arguments are not provided in the `args` slice, an error message is
/// returned as a `String`. The error message will specify the correct usage of the command.
///
fn parse_args(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        let error_message = "Usage: git_rm [options] <file>...";
        return Err(error_message.to_string());
    }

    Ok(())
}

/// Parses a single command-line option and extracts relevant information.
///
/// This function takes a command-line option as input and parses it to determine its
/// significance in the context of the `git_rm` program. It returns a tuple containing the
/// following information:
///
/// * The name of the option as a `String`.
/// * A boolean indicating whether directories should be removed recursively.
/// * A boolean indicating whether the option specifies removal from the index only.
/// * A boolean indicating whether the option enables a dry run (showing files to be removed
///   without actually removing them).
/// * A boolean indicating whether the option instructs to ignore non-matching files.
///
/// # Arguments
///
/// * `arg` - A command-line option as a `&str`.
///
/// # Returns
///
/// Returns a tuple containing the parsed information.
///
fn parse_option(arg: &str) -> (String, bool, bool, bool, bool) {
    let mut option_name = String::new();
    let mut remove_directories_recursively = false;
    let mut remove_from_index_only = false;
    let mut show_files_to_remove = false;
    let mut ignore_non_matching = false;

    match arg {
        "--" => {} // Separator between options and files
        "-f" | "--force" => {
            option_name = arg.to_string();
        }
        "-n" | "--dry-run" => {
            option_name = arg.to_string();
            show_files_to_remove = true;
        }
        "-r" => {
            remove_directories_recursively = true;
        }
        "--cached" => {
            option_name = arg.to_string();
            remove_from_index_only = true;
        }
        "--ignore-unmatch" => {
            option_name = arg.to_string();
            ignore_non_matching = true;
        }
        "-q" | "--quiet" => {
            option_name = arg.to_string();
        }
        _ => {
            eprintln!("Invalid option: {}", arg);
        }
    }

    (
        option_name,
        remove_directories_recursively,
        remove_from_index_only,
        show_files_to_remove,
        ignore_non_matching,
    )
}

/// Processes a list of files for removal based on specified options.
///
/// This function takes a list of file paths, along with boolean flags indicating whether
/// directories should be removed recursively and whether removal should be from the index
/// only. It processes each file for removal according to the specified options and prints
/// relevant information.
///
/// # Arguments
///
/// * `files` - A slice of `String` containing file paths to be removed.
/// * `remove_directories_recursively` - A boolean flag indicating whether directories
///   should be removed recursively.
/// * `remove_from_index_only` - A boolean flag indicating whether files should be removed
///   from the index only.
///
fn process_files_for_removal(
    files: &[String],
    remove_directories_recursively: bool,
    remove_from_index_only: bool,
) {
    for file in files {
        if remove_directories_recursively {
            let _ = remove_file_or_directory(file);
        } else if remove_from_index_only {
                match remove_file_from_index(file) {
                    Ok(_) => {
                        println!("Removed from index: {}", file);
                    }
                    Err(err) => {
                        eprintln!("Error removing from index: {}", err);
                    }
                }
            } else {
                match remove_file(file) {
                    Ok(_) => {
                        println!("Removed from file system: {}", file);
                    }
                    Err(err) => {
                        eprintln!("Error removing from file system: {}", err);
                    }
                }
            }
        }
    }


/// Handles command-line options and arguments for the git_rm utility.
///
/// This function checks the provided list of files and options, displaying relevant information
/// based on the specified options. If the provided list of files is empty or if the `--dry-run`
/// option is used, the function will not perform any removal operations and will return `false`.
///
/// # Arguments
///
/// * `files` - A reference to a `Vec<String>` containing the file paths to be removed.
/// * `show_files_to_remove` - A boolean flag indicating whether the `--dry-run` option is used
///   to display files that would be removed without actually removing them.
///
/// # Returns
///
/// A boolean value, `true` if removal operations should proceed, or `false` if no removal
/// operations should be executed.
///
fn handle_options_and_args(files: &Vec<String>, show_files_to_remove: bool) -> bool {
    if files.is_empty() {
        eprintln!("You must provide at least one file to remove.");
        return false;
    }
    // Process options
    if show_files_to_remove {
        println!("Files that would be removed:");
        for file in files {
            println!("{}", file);
        }
        return false;
    }
    true
}

/// Handles the behavior when the `--ignore-unmatch` option is used.
///
/// This function checks whether the `--ignore-unmatch` option is specified. If the option is
/// provided, the function does not display the list of removed files, ensuring that no errors
/// are shown when no matching files are found. If the option is not specified, the function will
/// display the list of removed files.
///
/// # Arguments
///
/// * `ignore_non_matching` - A boolean flag indicating whether the `--ignore-unmatch` option is
///   specified. If `true`, the function will not display the list of removed files when no
///   matching files are found. If `false`, the list of removed files will be displayed.
/// * `files` - A reference to a `Vec<String>` containing the file paths that were removed.
///
fn handle_ignore_non_matching(ignore_non_matching: bool, files: &Vec<String>) {
    if !ignore_non_matching {
        println!("Removed files:");
        for file in files {
            println!("{}", file);
        }
    }
}

/// Git rm-like command for removing files from the file system and/or index.
///
/// This function simulates the behavior of the Git `rm` command. It parses command-line arguments,
/// processes options, and removes files from the file system and/or index, depending on the provided
/// options.
///
/// # Arguments
///
/// * `args` - A vector of strings representing the command-line arguments.
///
pub fn git_rm() {
    let args: Vec<String> = env::args().collect();
    match parse_args(&args) {
        Ok(_) => {}
        Err(error_message) => {
            eprintln!("{}", error_message);
            return;
        }
    }
    let mut files: Vec<String> = Vec::new();
    let mut options: Vec<String> = Vec::new();
    let mut remove_directories_recursively = false;
    let mut remove_from_index_only = false;
    let mut show_files_to_remove = false;
    let mut ignore_non_matching = false;

    // Create an iterator to process command-line arguments, skipping the program name.
    let  iter = args.iter().skip(1);

    for arg in iter {
        if arg.starts_with('-') {
            let (option_name, remove_dirs, remove_index, show, ignore) = parse_option(arg.as_str());
            options.push(option_name);
            remove_directories_recursively |= remove_dirs;
            remove_from_index_only |= remove_index;
            show_files_to_remove |= show;
            ignore_non_matching |= ignore;
        } else {
            files.push(arg.to_string());
        }
    }
    handle_options_and_args(&files, show_files_to_remove);
    if remove_directories_recursively {
        process_files_for_removal(&files, true, remove_from_index_only);
    } else {
        process_files_for_removal(&files, false, remove_from_index_only);
    }
    handle_ignore_non_matching(ignore_non_matching, &files);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    /// Test the removal of an existing file.
    ///
    /// This test creates a temporary file, attempts to remove it using the `remove_file` function,
    /// and then asserts that the removal operation was successful and that the file no longer exists.
    #[test]
    fn test_remove_file_existing() {
        // Create a temporary file for testing.
        let file_name = "test_remove_file_existing.txt";
        let mut file = File::create(file_name).expect("Failed to create test file.");
        file.write_all(b"Test data")
            .expect("Failed to write to test file.");

        // Test the remove_file function.
        let result = remove_file(file_name);
        assert!(result.is_ok());

        // Ensure the file no longer exists.
        assert!(!Path::new(file_name).exists());
    }

    /// Test the removal of a non-existing file.
    ///
    /// This test attempts to remove a file that does not exist using the `remove_file` function.
    /// It then asserts that the removal operation results in an error with the "Not Found" kind.
    #[test]
    fn test_remove_file_non_existing() {
        // Test removing a non-existing file.
        let file_name = "non_existing_file.txt";

        // Test the remove_file function.
        let result = remove_file(file_name);
        assert!(result.is_err());

        // Ensure an error with "Not Found" kind.
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    /// Test the removal of a non-existing file from the Git index.
    ///
    /// This test attempts to remove a file from the Git index that does not exist using the
    /// `remove_file_from_index` function. It then asserts that the removal operation results in an
    /// error with the "Not Found" kind, which is expected when attempting to remove a non-existing file
    /// from the Git index.
    #[test]
    fn test_remove_file_from_index_non_existing() {
        // Test removing a non-existing file from the index.
        let file_name = "non_existing_file.txt";

        // Test the remove_file_from_index function.
        let result = remove_file_from_index(file_name);
        assert!(result.is_err());

        // Ensure an error with "Not Found" kind.
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    /// Test reading a 32-bit unsigned integer (u32) from a byte array.
    ///
    /// This test checks the correctness of the `read_u32` function by passing it a byte array
    /// `[0x01, 0x02, 0x03, 0x04]` representing a 32-bit integer with the value `16909060`.
    /// The test then asserts that the result obtained from the `read_u32` function matches the expected value.
    ///
    /// This test ensures that the `read_u32` function correctly interprets the given byte array and
    /// performs the necessary bit shifting and bitwise OR operations to obtain the expected 32-bit integer value.
    #[test]
    fn test_read_u32() {
        let bytes: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let result = read_u32(&bytes);
        assert_eq!(result, 16909060);
    }

    /// Test removing an existing file using the remove_file_or_directory function.
    ///
    /// This test creates a temporary file for testing, `test_remove_file_or_directory_existing_file.txt`.
    /// It then calls the `remove_file_or_directory` function to remove the file and checks if the
    /// operation was successful. The test asserts that the result of the removal operation is `Ok`, indicating success.
    ///
    /// Finally, the test ensures that the file no longer exists by checking whether the file path is absent
    /// using `Path::new(file_name).exists()`, which should return `false` after the file is removed.
    ///
    /// This test validates that the `remove_file_or_directory` function can successfully remove an existing file.
    #[test]
    fn test_remove_file_or_directory_existing_file() {
        // Create a temporary file for testing.
        let file_name = "test_remove_file_or_directory_existing_file.txt";
        let mut file = File::create(file_name).expect("Failed to create test file.");

        // Test the remove_file_or_directory function.
        let result = remove_file_or_directory(file_name);
        assert!(result.is_ok());

        // Ensure the file no longer exists.
        assert!(!Path::new(file_name).exists());
    }

    /// Test removing an existing directory using the remove_file_or_directory function.
    ///
    /// This test creates a temporary directory for testing, `test_remove_file_or_directory_existing_directory`.
    /// It then calls the `remove_file_or_directory` function to remove the directory and checks if the
    /// operation was successful. The test asserts that the result of the removal operation is `Ok`, indicating success.
    ///
    /// Finally, the test ensures that the directory no longer exists by checking whether the directory path
    /// is absent using `Path::new(dir_name).exists()`, which should return `false` after the directory is removed.
    ///
    /// This test validates that the `remove_file_or_directory` function can successfully remove an existing directory.
    #[test]
    fn test_remove_file_or_directory_existing_directory() {
        // Create a temporary directory for testing.
        let dir_name = "test_remove_file_or_directory_existing_directory";
        fs::create_dir(dir_name).expect("Failed to create test directory.");

        // Test the remove_file_or_directory function.
        let result = remove_file_or_directory(dir_name);
        assert!(result.is_ok());

        // Ensure the directory no longer exists.
        assert!(!Path::new(dir_name).exists());
    }

    /// Test removing a non-existing file or directory using the remove_file_or_directory function.
    ///
    /// This test is designed to check the behavior of the `remove_file_or_directory` function when
    /// attempting to remove a file or directory that does not exist. It starts by specifying the
    /// `name` variable with a path that doesn't correspond to an existing file or directory. The
    /// `remove_file_or_directory` function is then called with this non-existing path.
    ///
    /// The test asserts that the result of the removal operation is an error (`Err`) by using
    /// `assert!(result.is_err())`. This indicates that an error occurred, as expected. Further, the
    /// test checks that the error has the "Not Found" kind (i.e., `io::ErrorKind::NotFound`) by using
    /// `assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound)`. This ensures that the error
    /// indeed corresponds to a "Not Found" error, which is the expected error type when attempting
    /// to remove something that doesn't exist.
    ///
    /// This test validates that the `remove_file_or_directory` function correctly handles the case of
    /// attempting to remove a non-existing file or directory.
    #[test]
    fn test_remove_file_or_directory_non_existing() {
        // Test removing a non-existing file or directory.
        let name = "non_existing_path";

        // Test the remove_file_or_directory function.
        let result = remove_file_or_directory(name);
        assert!(result.is_err());

        // Ensure an error with "Not Found" kind.
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
