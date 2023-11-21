use std::io::{self, Error, ErrorKind};

use crate::config::Config;

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

    Ok(())
}
//Usage: git config set-user-info name email
