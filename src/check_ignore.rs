use std::io::{self, Write};

use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::ignorer::Ignorer;
use crate::logger::Logger;
use crate::utils::get_current_time;

/// Logs the 'git check-ignore' command with the specified parameters.
///
/// This function logs the 'git check-ignore' command with the provided parameters to a file named
/// 'logger_commands.txt'.
///
/// # Arguments
///
/// * `name_of_ignorer` - A string slice representing the name of the ignorer.
/// * `git_ignore_path` - A string slice representing the path to the Git ignore file.
/// * `option` - A string slice representing the option used in the check-ignore command.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_check_ignore(
    name_of_ignorer: &str,
    git_ignore_path: &str,
    option: &str,
) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git check-ignore': Name of Ignorer '{}', Git Ignore Path '{}', Option '{}', {}",
        name_of_ignorer,
        git_ignore_path,
        option,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Checks if specified paths should be ignored based on the provided `Ignorer`.
///
/// # Arguments
///
/// * `ignorer_name` - A string representing the name of the ignorer.
/// * `ignorer` - An instance of the `Ignorer` trait used for path exclusion.
/// * `line` - A vector of strings representing the input line, usually obtained from command line arguments.
/// * `output` - A mutable reference to an implementor of the `Write` trait for outputting results.
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. If there is an error, a `fatal: no path specified` message
/// is written to the `output` and an `io::Error` with kind `InvalidInput` is returned.
///
/// # Panics
///
/// This function may panic if the provided `Write` implementor encounters an error during writing.
///
pub fn git_check_ignore(
    ignorer_name: &str,
    git_ignore_path: &str,
    line: Vec<String>,
    output: &mut impl Write,
) -> io::Result<()> {
    log_check_ignore(ignorer_name, git_ignore_path, "check-ignore")?;

    let ignorer = Ignorer::load(git_ignore_path);

    if line.len() == 2 || (line.len() == 3 && line[2].eq("-v")) {
        writeln!(output, "fatal: no path specified")?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No path specified",
        ));
    } else if line[2].eq("-v") {
        let mut line_number = 1;
        for path in line.iter().skip(3) {
            if ignorer.ignore(path) {
                writeln!(output, "{}:{}:{}", ignorer_name, line_number, path)?;
            }
            line_number += 1;
        }
    } else if line[2].starts_with('-') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid option",
        ));
    } else {
        for path in line.iter().skip(2) {
            if ignorer.ignore(path) {
                writeln!(output, "{}", path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

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
    fn test_invalid_arguments() -> io::Result<()> {
        let test_directory = "tests/check_ignore_fake_repo_1";
        let gitignore_path = format!("{}/.mgitignore", test_directory);
        create_if_not_exists(test_directory, true)?;
        create_if_not_exists(&gitignore_path, false)?;

        let mut output: Vec<u8> = vec![];
        let command_line: Vec<String> = vec!["git".to_string(), "check-ignore".to_string()];
        let result = git_check_ignore(".mgitignore", &gitignore_path, command_line, &mut output);

        assert!(result.is_err());
        std::fs::remove_dir_all(test_directory)?;

        Ok(())
    }

    #[test]
    fn test_invalid_arguments_verbose_flag() -> io::Result<()> {
        let test_directory = "tests/check_ignore_fake_repo_2";
        let gitignore_path = format!("{}/.mgitignore", test_directory);
        create_if_not_exists(test_directory, true)?;
        create_if_not_exists(&gitignore_path, false)?;
        let mut output: Vec<u8> = vec![];
        let command_line: Vec<String> = vec![
            "git".to_string(),
            "check-ignore".to_string(),
            "-v".to_string(),
        ];

        let result = git_check_ignore(".mgitignore", &gitignore_path, command_line, &mut output);

        assert!(result.is_err());

        std::fs::remove_dir_all(test_directory)?;

        Ok(())
    }

    #[test]
    fn test_invalid_arguments_invalid_flag() -> io::Result<()> {
        let test_directory = "tests/check_ignore_fake_repo_3";
        let gitignore_path = format!("{}/.mgitignore", test_directory);
        create_if_not_exists(test_directory, true)?;
        create_if_not_exists(&gitignore_path, false)?;
        let mut output: Vec<u8> = vec![];
        let command_line: Vec<String> = vec![
            "git".to_string(),
            "check-ignore".to_string(),
            "-a".to_string(),
        ];

        let result = git_check_ignore(".mgitignore", &gitignore_path, command_line, &mut output);

        assert!(result.is_err());

        std::fs::remove_dir_all(test_directory)?;

        Ok(())
    }
    #[test]
    fn test_check_ignore_correct_arguments() -> io::Result<()> {
        let path = "tests/check_ignore_fake_repo_4";
        let gitignore_path = path.to_string() + "/.mgitignore";
        create_if_not_exists(path, true)?;
        let mut file = File::create(&gitignore_path)?;
        writeln!(file, "file")?;
        let mut output: Vec<u8> = vec![];
        let line: Vec<String> = vec![
            "git".to_string(),
            "check-ignore".to_string(),
            "file".to_string(),
        ];
        let result = git_check_ignore(".mgitignore", &gitignore_path, line, &mut output);
        assert!(result.is_ok());
        let result = String::from_utf8(output).expect("Invalid UTF-8");
        assert!(result.eq("file\n"));
        std::fs::remove_dir_all(path)?;

        Ok(())
    }

    #[test]
    fn test_check_ignore_correct_arguments_verbose_flag() -> io::Result<()> {
        let path = "tests/check_ignore_fake_repo_5";
        let gitignore_path = path.to_string() + "/.mgitignore";
        create_if_not_exists(path, true)?;
        let mut file = File::create(&gitignore_path)?;
        writeln!(file, "file")?;
        let mut output: Vec<u8> = vec![];
        let line: Vec<String> = vec![
            "git".to_string(),
            "check-ignore".to_string(),
            "-v".to_string(),
            "file".to_string(),
        ];
        let result = git_check_ignore(".mgitignore", &gitignore_path, line, &mut output);
        assert!(result.is_ok());
        let result = String::from_utf8(output).expect("Invalid UTF-8");
        assert!(result.contains("file\n"));
        std::fs::remove_dir_all(path)?;

        Ok(())
    }
}
