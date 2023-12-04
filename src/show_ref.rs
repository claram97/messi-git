use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use crate::utils::get_current_time;
use crate::{configuration::LOGGER_COMMANDS_FILE, logger::Logger};
use std::fs::File;
use std::io::prelude::*;

/// Logs the 'git show-ref' command with the specified Git directory and command line arguments.
///
/// This function logs the 'git show-ref' command with the provided Git directory and command line
/// arguments to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git directory.
/// * `line` - The command line arguments used with 'git show-ref'.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_show_ref(git_dir: &str, line: Vec<String>) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git show-ref': Git Directory '{}', Line '{:?}', {}",
        git_dir,
        line,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Displays information about Git references based on the provided command-line arguments.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
/// * `line` - A vector of strings representing the command-line arguments.
/// * `output` - A mutable reference to a type implementing the `Write` trait where the output will be written.
///
/// # Errors
///
/// Returns an `io::Result<()>`:
/// * `Ok(())` - If the operation succeeds.
/// * `Err(io::Error)` - If an I/O error occurs during the process.
///
/// This function serves as an entry point for displaying information about Git references. It delegates the
/// functionality to other functions based on the provided command-line arguments. If the number of arguments
/// is 2, it calls `show_ref` to display heads and tags. If the number of arguments is 3, it calls `show_ref_with_options`
/// to handle additional options like `--heads`, `--tags`, `--hash`, and `--verify`. For more than 3 arguments,
/// it checks for the `--verify` option and calls `verify_ref`. If an invalid option or an incorrect number of
/// arguments is specified, an `InvalidInput` error is returned.
///
/// Note: In case of any errors during the process, an `io::Error` is returned.
///
/// See also: [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// # Panics
///
/// This function may panic if it encounters unexpected issues while processing the command-line arguments.
///
/// # Safety
///
/// This function assumes that the provided Git directory path is valid and accessible, and the output
/// writer is correctly implemented.
///
pub fn git_show_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 2 {
        show_ref(git_dir, output)?;
    } else if line.len() == 3 {
        show_ref_with_options(git_dir, line.clone(), output)?;
    } else if line.len() >= 3 {
        if line[2].eq("--verify") {
            verify_ref(git_dir, line.clone(), output)?;
        } else if line[2].starts_with("--") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid option specified",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    log_show_ref(git_dir, line)?;
    Ok(())
}

/// Verifies and displays information about the specified Git references.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
/// * `line` - A vector of strings representing the command-line arguments.
/// * `output` - A mutable reference to a type implementing the `Write` trait where the output will be written.
///
/// # Errors
///
/// Returns an `io::Result<()>`:
/// * `Ok(())` - If the operation succeeds.
/// * `Err(io::Error)` - If an I/O error occurs during the process.
///
/// This function verifies the specified Git references by checking if they exist in the Git directory. For each reference
/// provided in the `line` vector, it prints the content and the reference name if the reference exists. If a reference
/// does not exist, it prints a fatal error message.
///
/// Note: In case of any errors during the process, an `io::Error` is returned.
///
/// See also: [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// # Panics
///
/// This function may panic if it encounters unexpected issues while processing the Git references.
///
/// # Safety
///
/// This function assumes that the provided Git directory path is valid and accessible, and the output
/// writer is correctly implemented.
///
fn verify_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    for line_path in line.iter().skip(3) {
        let path_to_verify = format!("{}/{}", git_dir, &line_path);
        let path = Path::new(&path_to_verify);

        if !path.exists() {
            writeln!(output, "fatal: '{}' - not a valid ref\n", &line_path)?;
        } else {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            writeln!(output, "{}\t{}\n", contents.trim(), &line_path)?;
        }
    }
    Ok(())
}

fn show_refs_in_remotes_folder(
    remotes_path: &str,
    is_hash: bool,
    output: &mut impl Write,
) -> io::Result<()> {
    let path = Path::new(&remotes_path);
    if path.exists() {
        for entry in fs::read_dir(remotes_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let string_path = path.to_string_lossy().to_string();
                let splitted: Vec<&str> = string_path.split("remotes").collect();
                let type_ = format!("{}{}", "remotes", splitted[1]);
                process_files_in_directory(
                    path.to_string_lossy().as_ref(),
                    &type_,
                    is_hash,
                    output,
                )?;
            }
        }
    }
    Ok(())
}

/// Shows references in the specified Git directory for heads and tags.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
/// * `output` - A mutable reference to a type implementing the `Write` trait where the output will be written.
///
/// # Errors
///
/// Returns an `io::Result<()>`:
/// * `Ok(())` - If the operation succeeds.
/// * `Err(io::Error)` - If an I/O error occurs during the process.
///
/// This function shows references in the specified Git directory for both heads and tags. It lists the references with
/// type information, printing the content and the associated type and file name for both heads and tags.
///
/// Note: In case of any errors during the process, an `io::Error` is returned.
///
/// See also: [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// # Panics
///
/// This function may panic if it encounters unexpected issues while processing the Git directory.
///
/// # Safety
///
/// This function assumes that the provided Git directory path is valid and accessible, and the output
/// writer is correctly implemented.
///
fn show_ref(git_dir: &str, output: &mut impl Write) -> io::Result<()> {
    let heads_path = format!("{}/{}", git_dir, "refs/heads");
    let tags_path = format!("{}/{}", git_dir, "refs/tags");
    let remotes_path = format!("{}/{}", git_dir, "refs/remotes");
    process_files_in_directory(&heads_path, "heads", false, output)?;
    process_files_in_directory(&tags_path, "tags", false, output)?;
    show_refs_in_remotes_folder(&remotes_path, false, output)?;

    Ok(())
}

/// Shows references in the specified Git directory based on the provided options.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
/// * `line` - A vector of strings representing the command-line options.
/// * `output` - A mutable reference to a type implementing the `Write` trait where the output will be written.
///
/// # Errors
///
/// Returns an `io::Result<()>`:
/// * `Ok(())` - If the operation succeeds.
/// * `Err(io::Error)` - If an I/O error occurs during the process.
///
/// This function shows references in the specified Git directory based on the provided options in the `line` vector.
/// The supported options include `--heads`, `--tags`, `--hash`, and `--verify`. For `--heads` and `--tags`, it lists
/// the references with type information. For `--hash`, it outputs only the hash content without type information.
/// `--verify` prints an error message indicating that it requires a reference. If an invalid option is specified, an
/// `InvalidInput` error is returned.
///
/// Note: In case of any errors during the process, an `io::Error` is returned.
///
/// See also: [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// # Panics
///
/// This function may panic if it encounters unexpected issues while processing the options.
///
/// # Safety
///
/// This function assumes that the provided Git directory path is valid and accessible, and the output
/// writer is correctly implemented.
///
fn show_ref_with_options(
    git_dir: &str,
    line: Vec<String>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line[2].eq("--heads") {
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let path = Path::new(&heads_path);
        if path.exists() {
            process_files_in_directory(&heads_path, "heads", false, output)?;
        }
    } else if line[2].eq("--tags") {
        let tags_path: String = format!("{}/{}", git_dir, "refs/tags");
        let path = Path::new(&tags_path);
        if path.exists() {
            process_files_in_directory(&tags_path, "tags", false, output)?;
        }
    } else if line[2].eq("--hash") {
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let remotes_path = format!("{}/{}", git_dir, "refs/remotes");
        let path = Path::new(&heads_path);
        if path.exists() {
            process_files_in_directory(&heads_path, "heads", true, output)?;
        }
        let path = Path::new(&tags_path);
        if path.exists() {
            process_files_in_directory(&tags_path, "tags", true, output)?;
        }

        show_refs_in_remotes_folder(&remotes_path, true, output)?;
    } else if line[2].eq("--verify") {
        writeln!(output, "fatal: --verify requires a reference")?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fatal: --verify requires a reference\n",
        ));
    } else if line[2].starts_with("--") {
        output.write_all("Invalid option specified".as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid option specified\n",
        ));
    }
    Ok(())
}

/// Processes files in the specified directory and writes their contents to the provided output.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the directory containing the files to be processed.
/// * `type_` - A string slice specifying the type associated with the files.
/// * `is_hash` - A boolean indicating whether to output only the contents without type information.
/// * `output` - A mutable reference to a type implementing the `Write` trait where the processed data will be written.
///
/// # Errors
///
/// Returns an `io::Result<()>`:
/// * `Ok(())` - If the operation succeeds.
/// * `Err(io::Error)` - If an I/O error occurs during file processing.
///
/// This function reads each file in the specified directory, extracts its contents, and writes the information
/// to the output. If `is_hash` is true, it only outputs the contents without type information; otherwise, it includes
/// the type and file name in the output.
///
/// Note: In case of any errors during file processing, an `io::Error` is returned.
///
/// See also: [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// # Panics
///
/// This function may panic if it encounters unexpected issues while processing the files.
///
/// # Safety
///
/// This function assumes that the provided directory path is valid and accessible, and the output
/// writer is correctly implemented.
///
fn process_files_in_directory(
    path: &str,
    type_: &str,
    is_hash: bool,
    output: &mut impl Write,
) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            let file_name = match file_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error"));
                }
            };

            let mut file = fs::File::open(&file_path)?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            if is_hash {
                writeln!(output, "{}\n", contents.trim())?;
            } else {
                writeln!(
                    output,
                    "{}\trefs/{}/{}\n",
                    contents.trim(),
                    type_,
                    file_name
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_verify_ref_invalid_args() -> io::Result<()> {
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--verify".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        let result = show_ref_with_options("git_dir", line, &mut output);
        assert!(result.is_err());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("fatal"));
        Ok(())
    }

    #[test]
    fn test_invalid_args() -> io::Result<()> {
        let line = vec!["git".to_string(), "show-ref".to_string(), "--a".to_string()];
        let mut output: Vec<u8> = vec![];
        let result = show_ref_with_options("git_dir", line, &mut output);
        assert!(result.is_err());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("Invalid"));
        Ok(())
    }

    #[test]
    fn test_verify_ref_invalid_ref() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_2";
        let git_dir = format!("{}/{}", path, ".mgit");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--verify".to_string(),
            "refs/heads/some_ref".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        verify_ref(&git_dir, line, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("fatal"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_verify_ref_finds_the_correct_reference() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_3";
        let git_dir = format!("{}/{}", path, ".mgit");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&head_ref, false)?;
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--verify".to_string(),
            "refs/heads/some_ref".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        verify_ref(&git_dir, line, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("refs/heads/some_ref"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_show_ref_shows_all_heads_remotes_and_tags() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_4";
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags = format!("{}/{}", git_dir, "refs/tags");
        let remotes = format!("{}/{}", git_dir, "refs/remotes");
        let remote_origin = format!("{}/{}", git_dir, "refs/remotes/origin");
        let remote_base = format!("{}/{}", git_dir, "refs/remotes/base");
        let remote_origin_ref1 = format!("{}/{}", git_dir, "refs/remotes/origin/ref1");
        let remote_origin_ref2 = format!("{}/{}", git_dir, "refs/remotes/origin/ref2");
        let remote_base_ref = format!("{}/{}", git_dir, "refs/remotes/base/ref");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags, true)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        create_if_not_exists(&remotes, true)?;
        create_if_not_exists(&remote_origin, true)?;
        create_if_not_exists(&remote_base, true)?;
        create_if_not_exists(&remote_origin_ref1, false)?;
        create_if_not_exists(&remote_origin_ref2, false)?;
        create_if_not_exists(&remote_base_ref, false)?;
        let mut output: Vec<u8> = vec![];
        show_ref(&git_dir, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("refs/heads/some_ref"));
        assert!(output_string.contains("refs/tags/some_tag"));
        assert!(output_string.contains("refs/remotes/origin/ref1"));
        assert!(output_string.contains("refs/remotes/origin/ref2"));
        assert!(output_string.contains("refs/remotes/base/ref"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_show_heads_ref_shows_only_heads_refs() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_5";
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags = format!("{}/{}", git_dir, "refs/tags");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags, true)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--heads".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        show_ref_with_options(&git_dir, line, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("refs/heads/some_ref"));
        assert!(!output_string.contains("refs/tags/some_tag"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_show_tags_ref_shows_only_tags_refs() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_6";
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags = format!("{}/{}", git_dir, "refs/tags");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags, true)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--tags".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        show_ref_with_options(&git_dir, line, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(!output_string.contains("refs/heads/some_ref"));
        assert!(output_string.contains("refs/tags/some_tag"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    fn write_to_file(file_path: &str, content: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_show_hashes_refs_shows_only_hashes_from_all_tags_heads_and_remotes() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_7";
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags = format!("{}/{}", git_dir, "refs/tags");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        create_if_not_exists(path, true)?;
        let remotes_path = format!("{}/{}", git_dir, "refs/remotes");
        let origin_folder = format!("{}/{}", git_dir, "refs/remotes/origin");
        let origin_ref = format!("{}/{}", git_dir, "refs/remotes/origin/branch");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags, true)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        create_if_not_exists(&remotes_path, true)?;
        create_if_not_exists(&origin_folder, true)?;
        create_if_not_exists(&origin_ref, false)?;
        write_to_file(&origin_ref, "7891")?;
        write_to_file(&head_ref, "1234")?;
        write_to_file(&tag_ref, "4567")?;
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--hash".to_string(),
        ];
        let mut output: Vec<u8> = vec![];
        show_ref_with_options(&git_dir, line, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("7891"));
        assert!(output_string.contains("1234"));
        assert!(output_string.contains("4567"));
        assert!(!output_string.contains("refs/heads/some_ref"));
        assert!(!output_string.contains("refs/remotes/origin/branch"));
        assert!(!output_string.contains("refs/tags/some_tag"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_process_files_in_directory_correct_function() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_1";
        let git_dir = format!("{}/{}", path, ".mgit");
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let remotes_path = format!("{}/{}", git_dir, "refs/remotes");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        let origin_folder = format!("{}/{}", git_dir, "refs/remotes/origin");
        let origin_ref = format!("{}/{}", git_dir, "refs/remotes/origin/branch");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags_path, true)?;
        create_if_not_exists(&remotes_path, true)?;
        create_if_not_exists(&origin_folder, true)?;
        create_if_not_exists(&origin_ref, false)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        write_to_file(&head_ref, "1234")?;
        write_to_file(&tag_ref, "4567")?;
        write_to_file(&origin_ref, "7891")?;
        let mut output: Vec<u8> = vec![];
        let result = process_files_in_directory(&heads_path, "heads", false, &mut output);
        assert!(result.is_ok());
        let result = process_files_in_directory(&tags_path, "tags", false, &mut output);
        assert!(result.is_ok());
        // let result = process_files_in_directory(&remotes_path, "remotes/origin", false, &mut output);
        // assert!(result.is_ok());
        let result = show_refs_in_remotes_folder(&remotes_path, false, &mut output);
        assert!(result.is_ok());
        let output_string = String::from_utf8(output).unwrap();
        let mut file = File::create("tests/test.txt")?;
        file.write_all(output_string.as_bytes())?;
        assert!(output_string.contains("1234"));
        assert!(output_string.contains("4567"));
        assert!(output_string.contains("7891"));
        assert!(output_string.contains("refs/heads/some_ref"));
        assert!(output_string.contains("refs/tags/some_tag"));
        assert!(output_string.contains("refs/remotes/origin/branch"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_process_files_in_directory_correct_function_hash_option() -> io::Result<()> {
        let path = "tests/show_ref_fake_repo_8";
        let git_dir = format!("{}/{}", path, ".mgit");
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let head_ref = format!("{}/{}", git_dir, "refs/heads/some_ref");
        let tag_ref = format!("{}/{}", git_dir, "refs/tags/some_tag");
        let remotes_path = format!("{}/{}", git_dir, "refs/remotes");
        let origin_folder = format!("{}/{}", git_dir, "refs/remotes/origin");
        let origin_ref = format!("{}/{}", git_dir, "refs/remotes/origin/branch");
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        create_if_not_exists(&tags_path, true)?;
        create_if_not_exists(&head_ref, false)?;
        create_if_not_exists(&tag_ref, false)?;
        create_if_not_exists(&remotes_path, true)?;
        create_if_not_exists(&origin_folder, true)?;
        create_if_not_exists(&origin_ref, false)?;
        write_to_file(&head_ref, "1234")?;
        write_to_file(&tag_ref, "4567")?;
        write_to_file(&origin_ref, "7891")?;
        let mut output: Vec<u8> = vec![];
        let result = process_files_in_directory(&heads_path, "heads", true, &mut output);
        assert!(result.is_ok());
        let result = process_files_in_directory(&tags_path, "tags", true, &mut output);
        assert!(result.is_ok());
        let result = show_refs_in_remotes_folder(&remotes_path, true, &mut output);
        assert!(result.is_ok());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("7891"));
        assert!(output_string.contains("1234"));
        assert!(output_string.contains("4567"));
        assert!(!output_string.contains("refs/heads/some_ref"));
        assert!(!output_string.contains("refs/tags/some_tag"));
        assert!(!output_string.contains("refs/remotes/origin/branch"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_git_show_ref_verify_without_path_returns_error() -> io::Result<()> {
        let mut output: Vec<u8> = vec![];
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--verify".to_string(),
        ];
        let result = git_show_ref("git_dir", line, &mut output);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_git_show_ref_unknown_arguments_returns_error() -> io::Result<()> {
        let mut output: Vec<u8> = vec![];
        let line = vec!["git".to_string(), "show-ref".to_string(), "--a".to_string()];
        let result = git_show_ref("git_dir", line, &mut output);
        assert!(result.is_err());
        Ok(())
    }
}
