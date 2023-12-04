use std::io::{self, Write};

use crate::config::Config;
use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::logger::Logger;
use crate::utils::get_current_time;

/// Logs the 'git remote' command with the specified subcommand.
///
/// This function logs the 'git remote' command with the provided subcommand to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `line` - A slice containing the subcommand and its arguments.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_remote(line: &[&str]) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git remote': Subcommand '{:?}', {}",
        line,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Performs operations related to remotes in the configuration.
///
/// This function allows adding, removing, changing the URL, and obtaining information about
/// remote repositories in the configuration. The configuration should be loaded from a file
/// that includes the basic data of an initial configuration. The available operations are as follows:
///
/// - `add`: Add a remote repository. Three arguments are required: "add name url".
/// - `remove`: Remove a remote repository. One argument is required: "remove name".
/// - `set-url`: Change the URL of a remote repository. Three arguments are required: "set-url name new-url".
/// - `get-url`: Get the URL of a remote repository. One argument is required: "get-url name".
/// - `rename`: Change the name of a remote repository. Two arguments are required: "rename new-name old-name".
///
/// # Arguments
///
/// - `config`: A mutable reference to the configuration to be modified. It requires to be uploaded before calling the function, and the initial config file should be valid (this meaning, it contains the basic data provided when you run git init). Otherwise, it can behave weirdly.
/// - `line`: A vector of strings containing the command line split into arguments.
/// - `output`: An object implementing the `Write` trait where the results or errors will be written.
///
pub fn git_remote(config: &mut Config, line: Vec<&str>, output: &mut impl Write) -> io::Result<()> {
    if line.is_empty() || line.len() > 3 {
        return report_error(output, "Invalid arguments.");
    }
    log_remote(&line)?;
    match line[0] {
        "remote" => handle_list_command(config, output),
        "add" => handle_add_command(config, &line, output),
        "remove" => handle_remove_command(config, &line, output),
        "set-url" => handle_set_url_command(config, &line, output),
        "get-url" => handle_get_url_command(config, &line, output),
        "rename" => handle_rename_command(config, &line, output),
        _ => report_error(output, &format!("error: Unknown subcommand {}\n", line[0])),
    }
}

/// Writes an error message to the specified output and returns an error.
///
/// This function writes the provided error message to the output, typically a writable stream,
/// and creates an error with the same message. It is used to report errors in the `git_remote`
/// function and its subcommand handlers.
///
/// # Arguments
///
/// - `output`: A mutable reference to an object implementing the `Write` trait, where the error message
///   will be written.
/// - `error_message`: A string representing the error message to report.
///
/// # Returns
///
/// Returns an error of type `io::Error` with the specified error message.
///
fn report_error(output: &mut impl Write, error_message: &str) -> io::Result<()> {
    output.write_all(error_message.as_bytes())?;
    Err(io::Error::new(io::ErrorKind::InvalidInput, error_message))
}

fn handle_list_command(config: &mut Config, output: &mut impl Write) -> io::Result<()> {
    config.list_remotes(output)?;
    Ok(())
}
/// Handles the "add" subcommand for Git remotes.
///
/// This function processes the "add" subcommand for Git remotes, validates the arguments, and
/// adds a new remote to the Git configuration if the arguments are valid.
///
/// # Arguments
///
/// - `config`: A mutable reference to the Git configuration.
/// - `line`: A reference to a vector of string slices representing the subcommand and its arguments.
/// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
///   or output will be written.
///
/// # Returns
///
/// Returns `Ok(())` if the "add" subcommand is successfully executed. If invalid arguments are provided,
/// it returns an error with an error message.
///
fn handle_add_command(
    config: &mut Config,
    line: &Vec<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() != 3 {
        return report_error(output, "Invalid arguments.");
    }
    let fetch = format!("+refs/heads/*:refs/remotes/{}/name/*", line[1]);
    config.add_remote(line[1].to_string(), line[2].to_string(), fetch, output)
}

/// Handles the "remove" subcommand for Git remotes.
///
/// This function processes the "remove" subcommand for Git remotes, validates the arguments, and
/// removes a remote from the Git configuration if the arguments are valid.
///
/// # Arguments
///
/// - `config`: A mutable reference to the Git configuration.
/// - `line`: A reference to a vector of string slices representing the subcommand and its arguments.
/// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
///   or output will be written.
///
/// # Returns
///
/// Returns `Ok(())` if the "remove" subcommand is successfully executed. If invalid arguments are provided,
/// it returns an error with an error message.
///
fn handle_remove_command(
    config: &mut Config,
    line: &Vec<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() != 2 {
        return report_error(output, "Invalid arguments.");
    }
    config.remove_remote(line[1], output)
}

/// Handles the "set-url" subcommand for Git remotes.
///
/// This function processes the "set-url" subcommand for Git remotes, validates the arguments, and
/// sets a new URL for an existing remote in the Git configuration if the arguments are valid.
///
/// # Arguments
///
/// - `config`: A mutable reference to the Git configuration.
/// - `line`: A reference to a vector of string slices representing the subcommand and its arguments.
/// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
///   or output will be written.
///
/// # Returns
///
/// Returns `Ok(())` if the "set-url" subcommand is successfully executed. If invalid arguments are provided,
/// it returns an error with an error message.
///
fn handle_set_url_command(
    config: &mut Config,
    line: &Vec<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() != 3 {
        return report_error(output, "Invalid arguments.");
    }
    config.set_url(line[1], line[2], output)
}

/// Handles the "get-url" subcommand for Git remotes.
///
/// This function processes the "get-url" subcommand for Git remotes, validates the arguments, and
/// retrieves and writes the URL of an existing remote to the specified output.
///
/// # Arguments
///
/// - `config`: A mutable reference to the Git configuration.
/// - `line`: A reference to a vector of string slices representing the subcommand and its arguments.
/// - `output`: A mutable reference to an object implementing the `Write` trait, where the remote's URL
///   or error message will be written.
///
/// # Returns
///
/// Returns `Ok(())` if the "get-url" subcommand is successfully executed. If invalid arguments are provided
/// or the specified remote doesn't exist, it returns an error with an error message.
///
fn handle_get_url_command(
    config: &mut Config,
    line: &Vec<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() != 2 {
        return report_error(output, "Invalid arguments.");
    }
    config.get_url(line[1], output)?;
    Ok(())
}

/// Handles the "rename" subcommand for Git remotes.
///
/// This function processes the "rename" subcommand for Git remotes, validates the arguments, and
/// renames an existing remote in the Git configuration if the arguments are valid.
///
/// # Arguments
///
/// - `config`: A mutable reference to the Git configuration.
/// - `line`: A reference to a vector of string slices representing the subcommand and its arguments.
/// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
///   or output will be written.
///
/// # Returns
///
/// Returns `Ok(())` if the "rename" subcommand is successfully executed. If invalid arguments are provided,
/// a remote with the new name already exists, or the specified remote doesn't exist, it returns an error
/// with an error message.
///
fn handle_rename_command(
    config: &mut Config,
    line: &Vec<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line.len() != 3 {
        return report_error(output, "Invalid arguments.");
    }
    config.change_remote_name(line[1], line[2], output)
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{fs::File, path::Path};

    use crate::{configuration::GIT_DIR_FOR_TEST, init};

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
    fn test_invalid_arguments_lenght() -> io::Result<()> {
        let line = vec!["", "", "", ""];
        let mut output: Vec<u8> = vec![];
        create_if_not_exists("tests/remote_fake_repo_0", true)?;
        create_if_not_exists("tests/remote_fake_repo_0/.mgit", true)?;
        create_if_not_exists("tests/remote_fake_repo_0/.mgit/config", false)?;
        let git_dir_path = "tests/remote_fake_repo_0/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_0")?;
        Ok(())
    }

    #[test]
    fn test_uknown_command_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_1", true)?;
        init::git_init(
            "tests/remote_fake_repo_1",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists("tests/remote_fake_repo_1/.mgit/config", false)?;
        let line = vec!["something"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_1/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_1")?;
        Ok(())
    }

    #[test]
    fn test_invalid_add_command_with_few_arguments_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_2", true)?;
        init::git_init(
            "tests/remote_fake_repo_2",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists("tests/remote_fake_repo_2/.mgit/config", false)?;
        let line = vec!["add"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_2/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_2")?;
        Ok(())
    }

    #[test]
    fn test_invalid_add_command_with_too_many_args_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_3", true)?;
        init::git_init(
            "tests/remote_fake_repo_3",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists("tests/remote_fake_repo_3/.mgit/config", false)?;
        let line = vec!["add", "new_remote", "url", "something else"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_3/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_3")?;
        Ok(())
    }

    #[test]
    fn test_valid_add_command_returns_ok() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_4", true)?;
        init::git_init(
            "tests/remote_fake_repo_4",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create("tests/remote_fake_repo_4/.mgit/config")?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["add", "new_remote", "url"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_4/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_ok());
        std::fs::remove_dir_all("tests/remote_fake_repo_4")?;
        Ok(())
    }

    #[test]
    fn test_invalid_remove_command_with_few_arguments_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_5", true)?;
        init::git_init(
            "tests/remote_fake_repo_5",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create("tests/remote_fake_repo_5/.mgit/config")?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["remove", "remote_name", "something else"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_5/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_5")?;
        Ok(())
    }

    #[test]
    fn test_invalid_add_command_with_too_many_arguments_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_6", true)?;
        init::git_init(
            "tests/remote_fake_repo_6",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create("tests/remote_fake_repo_6/.mgit/config")?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["remove"];
        let mut output: Vec<u8> = vec![];
        let git_dir_path = "tests/remote_fake_repo_6/.mgit";
        let mut config = Config::load(git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_6")?;
        Ok(())
    }

    #[test]
    fn test_set_url_command_with_few_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_7";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["set-url"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_set_url_command_with_too_many_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_8";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["set-url", "remote", "url", "extra_arg"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_get_url_command_with_few_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_9";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["get-url"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_get_url_command_with_too_many_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_10";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["get-url", "remote", "extra_arg"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_rename_command_with_few_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_11";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["rename"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_rename_command_with_too_many_args_makes_git_remote_fail() -> io::Result<()> {
        let path = "tests/remote_fake_repo_12";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let line = vec!["rename", "remote", "new_name", "extra_arg"];
        let mut output: Vec<u8> = vec![];
        let mut config = Config::load(&git_dir_path)?;
        let result = git_remote(&mut config, line, &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_valid_remove_command_returns_ok() -> io::Result<()> {
        let path = "tests/remote_fake_repo_13";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let mut config = Config::load(&git_dir_path)?;

        let line = vec!["add", "new_remote", "url"];
        let mut output: Vec<u8> = vec![];
        let _result = git_remote(&mut config, line, &mut output);

        let remove_line = vec!["remove", "new_remote"];
        let mut new_output: Vec<u8> = vec![];
        let result = git_remote(&mut config, remove_line, &mut new_output);
        assert!(result.is_ok());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_valid_rename_command_returns_ok() -> io::Result<()> {
        let path = "tests/remote_fake_repo_14";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let mut config = Config::load(&git_dir_path)?;

        let line = vec!["add", "new_remote", "url"];
        let mut output: Vec<u8> = vec![];
        let _result = git_remote(&mut config, line, &mut output);

        let remove_line = vec!["rename", "new_remote", "remote"];
        let mut new_output: Vec<u8> = vec![];
        let result = git_remote(&mut config, remove_line, &mut new_output);
        assert!(result.is_ok());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_valid_set_url_command_returns_ok() -> io::Result<()> {
        let path = "tests/remote_fake_repo_15";
        let config_path = path.to_string() + "/.mgit/config";
        let git_dir_path = path.to_string() + "/.mgit";
        create_if_not_exists(path, true)?;
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let config_data = format!("[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(config_data.as_bytes())?;
        config_file.flush()?;
        let mut config = Config::load(&git_dir_path)?;

        let line = vec!["add", "new_remote", "url"];
        let mut output: Vec<u8> = vec![];
        let _result = git_remote(&mut config, line, &mut output);

        let remove_line = vec!["set-url", "new_remote", "new_url"];
        let mut new_output: Vec<u8> = vec![];
        let result = git_remote(&mut config, remove_line, &mut new_output);
        assert!(result.is_ok());
        std::fs::remove_dir_all(path)?;
        Ok(())
    }
}
