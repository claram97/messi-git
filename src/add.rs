use std::fs;
use std::io;

use crate::configuration::GIT_DIR;
use crate::configuration::GIT_IGNORE;
use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::ignorer::is_subpath;
use crate::index::Index;
use crate::logger::Logger;
use crate::utils::get_current_time;
use std::path::Path;
use std::path::PathBuf;

use std::io::Write;

/// Process a file or directory specified by `file_name` and update the index accordingly.
///
/// If the path represented by `file_name` is a directory, add all files inside it to the index.
/// If it's a file, add only that file to the index.
///
/// # Arguments
///
/// * `index` - A mutable reference to an `Index` that represents the index of the Git repository.
/// * `file_name` - A string representing the path of the file or directory to be processed.
///
pub fn process_file_name(index: &mut Index, file_name: &str) -> io::Result<()> {
    if fs::metadata(file_name)?.is_dir() {
        for entry in fs::read_dir(file_name)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                let file_path_str = &file_path.to_string_lossy().to_string();
                match index.add_path(file_path_str) {
                    Ok(_) => {}
                    Err(error) => {
                        if error
                            .to_string()
                            .contains("The path is ignored by ignore file")
                        {
                        } else {
                            return Err(error);
                        }
                    }
                }
            }
        }
    } else {
        match index.add_path(file_name) {
            Ok(_) => {}
            Err(error) => {
                if error
                    .to_string()
                    .contains("The path is ignored by ignore file")
                {
                } else {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}

/// Logs a 'git add' command with the specified parameters.
///
/// This function logs a 'git add' command with the provided parameters to a file named
/// 'logger_commands.txt'.
///
/// # Arguments
///
/// * `add_type` - A string slice representing the type of addition (e.g., "new" or "modified").
/// * `file` - A string slice representing the path to the file being added.
/// * `git_dir` - A `Path` representing the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
fn log_add(add_type: &str, file: &str, _git_dir: &Path) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git add': Add type: {}, File: {}, Time: {}",
        add_type,
        file,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Add files to the Git index.
///
/// This function adds files to the Git index based on the provided path.
/// If the path points to a directory, all files inside the directory will be added.
/// If any file does not exist in the working directory, it will be removed from the index.
/// If the file neither exists in the index, an error is returned.
///
/// Files inside the repository directory will not be included.
/// TODO: .gitignore
///
/// IO errors may occur during IO operations. In those cases, an `Error` will be returned.
pub fn add(
    path: &str,
    index_path: &str,
    git_dir_path: &str,
    gitignore_path: &str,
    options: Option<Vec<String>>,
) -> io::Result<()> {
    if is_subpath(path, GIT_DIR) {
        return Ok(());
    }
    if let Some(params) = options {
        if params.len() == 1 && params[0] == "." {
            let current_dir = std::env::current_dir()?;
            let current_dir_str = &current_dir.to_string_lossy().to_string();
            if !current_dir_str.starts_with(GIT_DIR) {
                let file_names: Vec<String> = fs::read_dir(current_dir_str)?
                    .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
                    .collect();

                for file_name in file_names {
                    if file_name.eq(GIT_IGNORE)
                        || (!file_name.eq(GIT_IGNORE) && !file_name.starts_with(GIT_DIR))
                    {
                        let mut index = Index::load(index_path, git_dir_path, gitignore_path)?;
                        process_file_name(&mut index, &file_name)?;
                        index.write_file()?;

                        log_add("all", &file_name, &PathBuf::from(&path))?;
                    }
                }
            }
        }
    } else if !path.starts_with(GIT_DIR) {
        let mut index = Index::load(index_path, git_dir_path, gitignore_path)?;
        process_file_name(&mut index, path)?;
        index.write_file()?;
        log_add("single", path, &PathBuf::from(&path))?;
    }

    Ok(())
}
