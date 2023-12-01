use std::io::{self, Error, ErrorKind};

use crate::config::Config;
use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::logger::Logger;
use crate::utils::get_current_time;
use std::io::Write;

/// Logs the 'git config' command with the specified Git directory and command line.
///
/// This function logs the 'git config' command with the provided Git directory and
/// command line arguments to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
/// * `line` - A vector of strings representing the command line arguments.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_config(git_dir: &str, line: &[String]) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git config': Git Directory '{}', Command Line '{:?}', {}",
        git_dir,
        line,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Set Git user information (name and email) in the specified configuration file.
///
/// # Arguments
///
/// * `config_path` - A string slice representing the path to the Git configuration file.
/// * `line` - A vector of strings representing the command line arguments.
///
/// Usage: git config set-user-info "name" "email"
/// # Errors
///
/// The function returns an `io::Result` indicating whether setting the user information was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * Incorrect number of command line arguments, leading to an `InvalidInput` error.
/// * Unable to load the Git configuration, resulting in a `Config::load` error.
/// * Unable to set the user name and email in the configuration, leading to a `Config::set_user_name_and_email` error.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors.
pub fn git_config(git_dir: &str, line: Vec<String>) -> io::Result<()> {
    if line.len() != 5 && line.len() != 3 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Correct usage: git config set-user-info \"name\" \"email\" or git config get-user-info".to_string(),
        ));
    }
    let config = Config::load(git_dir)?;
    if line.len() == 5 {
        config.set_user_name_and_email(&line[3], &line[4])?;
    } else if line.len() == 3 {
        let (name, email) = config.get_user_name_and_email()?;
        println!("Name = {name}\nEmail = {email}")
    }
    log_config(git_dir, &line)?;
    Ok(())
}
//Usage: git config set-user-info name email
