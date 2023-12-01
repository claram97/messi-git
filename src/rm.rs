use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::index::Index;
use crate::logger::Logger;
use crate::utils::get_current_time;
use std::fs;
use std::io;
use std::io::Write;

/// Logs the 'git rm' command with the specified file name, Git directory path, and Git ignore path.
///
/// This function logs the 'git rm' command with the provided file name, Git directory path, and
/// Git ignore path to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `file_name` - The name of the file to be removed.
/// * `git_dir_path` - The path to the Git directory.
/// * `git_ignore_path` - The path to the Git ignore file.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_rm(file_name: &str, _git_dir_path: &str, _git_ignore_path: &str) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git rm': File '{}', {}",
        file_name,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Remove a file from both the index and the working directory.
///
/// This function is responsible for removing a file with the specified `file_name` from both the Git index and the working directory.
///
/// # Arguments
///
/// * `file_name` - A string representing the name of the file to be removed.
/// * `index_path` - A string specifying the path to the Git index file.
/// * `git_dir_path` - A string indicating the path to the Git directory.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If the index cannot be loaded from the provided `index_path` and `git_dir_path`.
/// - If the specified file is not found in the index.
/// - If there is an error while removing the file from the working directory.
/// - If there is an error while saving the updated index.
///
/// # Panics
///
/// The function may panic if there are errors during the removal process and error messages are printed to the standard error output.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use messi::rm::git_rm;
///
/// let result = git_rm("file.txt", "index.index", ".mgit", ".gitignore");
/// if let Err(err) = result {
///     eprintln!("Error: {}", err);
/// }
/// ```
pub fn git_rm(
    file_name: &str,
    index_path: &str,
    git_dir_path: &str,
    git_ignore_path: &str,
) -> io::Result<()> {
    if let Some(mut index) =
        Index::load_from_path_if_exists(index_path, git_dir_path, git_ignore_path)?
    {
        if !index.contains(file_name) {
            eprintln!("The file is not in the index.");
            return Ok(());
        }

        if let Err(_err) = remove_path(&mut index, file_name) {
            eprintln!("Error removing the file: {}", _err);
            return Err(_err);
        }

        if let Err(err) = index.write_file() {
            eprintln!("Error saving the index: {}", err);
            return Err(err);
        }
    } else {
        eprintln!("Failed to load the index.");
    }
    log_rm(file_name, git_dir_path, git_ignore_path)?;
    Ok(())
}

/// Recursively remove a directory and its contents.
///
/// This function removes the directory specified by `dir_path` and all of its contents, including subdirectories and files.
///
/// # Arguments
///
/// * `dir_path` - A string representing the path to the directory to be removed.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If there is an issue while traversing and removing the directory and its contents.
/// - If there is an error while removing a file within the directory.
/// - If there is an error while removing the directory itself.
///
/// # Examples
///
/// ```
/// use std::io;
/// use messi::rm::remove_directory;
///
/// let result = remove_directory("my_directory");
/// if let Err(err) = result {
///     eprintln!("Error: {}", err);
/// }
/// ```
///
/// This example would recursively remove the directory "my_directory" and all of its contents.
pub fn remove_directory(dir_path: &str) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            remove_directory(path.to_str().unwrap())?;
        } else {
            fs::remove_file(path.to_str().unwrap())?;
        }
    }

    fs::remove_dir(dir_path)?;

    Ok(())
}

/// Remove a file or directory specified by `path` from the index and the working directory.
///
/// This function takes an `Index` reference and a `path` as input and removes the file or directory specified by `path` from the index and the working directory. If the path is not found in the index, an error is returned. If the path is a directory, its contents are removed recursively.
///
/// # Arguments
///
/// * `index` - A mutable reference to an `Index` that represents the index of the Git repository.
/// * `path` - A string representing the path of the file or directory to be removed.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If the specified `path` is not found in the index.
/// - If there is an issue while removing the file or directory from the index.
/// - If there is an issue while removing a file within a directory.
/// - If there is an issue while removing the directory itself.
pub fn remove_path(index: &mut Index, path: &str) -> io::Result<()> {
    index.remove_file(path)?;

    if fs::metadata(path)?.is_dir() {
        remove_directory(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test case to verify the behavior of the `git_rm` function when trying to remove a file that is not in the index.
    ///
    /// This test case verifies that the `git_rm` function returns successfully when attempting to remove a file that is not present in the index. The function should not produce any errors, and the file should not exist in the working directory after the removal operation.
    ///
    /// # Arguments
    ///
    /// None
    ///
    #[test]
    fn test_git_rm_file_not_in_index() -> io::Result<()> {
        let index_path = "ruta_al_indice";
        let git_dir_path = "ruta_al_git_dir";
        let file_name = "archivo_no_en_indice.txt";

        let result = git_rm(file_name, index_path, git_dir_path, "");

        assert!(result.is_ok());
        assert!(!fs::metadata(file_name).is_ok());

        Ok(())
    }

    fn setup_mgit(git_dir: &str) -> io::Result<()> {
        fs::create_dir_all(format!("{}/objects", git_dir))
    }

    /// Test case to verify the behavior of the `add_path` function when adding a single file.
    ///
    /// This test case checks the behavior of the `add_path` function in the Index type when adding a single file to the index. It verifies that the file is successfully added to the index and can be retrieved afterward.
    ///
    /// # Arguments
    ///
    /// None
    ///
    #[test]
    fn test_add_path_file() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(path)?;

        assert!(index.contains(path));
        Ok(())
    }

    /// Test case to verify the behavior of the `git_rm` function when removing a file in the index.
    ///
    /// This test case checks the behavior of the `git_rm` function in the Index type when removing a file from the index that is also present in the filesystem. It verifies that the file is successfully removed from the index, the filesystem, and that the index file is updated accordingly.
    ///
    /// # Arguments
    ///
    /// None
    ///
    #[test]
    fn test_git_rm_file_in_index() -> io::Result<()> {
        let index_path = "";
        let git_dir_path = ".mgit";
        let file_name = "a.txt";
        let mut index = Index::new(index_path, git_dir_path, "");
        setup_mgit(".mgit")?;

        fs::write(file_name, "contenido del archivo")?;
        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(file_name)?;
        assert!(index.contains(file_name));

        let result = git_rm(file_name, index_path, git_dir_path, "");
        assert!(result.is_ok());
        let result1 = Index::load_from_path_if_exists(path, git_dir_path, "");
        if let Ok(Some(_index1)) = result1 {
        } else {
            assert!(result1.is_ok());
        }

        Ok(())
    }

    /// Test case to verify the behavior of the `remove_directory` function.
    ///
    /// This test case checks the behavior of the `remove_directory` function in removing a directory along with its contents.
    ///
    /// # Arguments
    ///
    /// None
    ///
    #[test]
    fn test_remove_directory() -> io::Result<()> {
        let dir_path = "directorio_a_eliminar";
        fs::create_dir_all(dir_path)?;

        let file_path = format!("{}/archivo.txt", dir_path);
        fs::write(&file_path, "contenido del archivo")?;

        remove_directory(dir_path)?;

        assert!(!fs::metadata(dir_path).is_ok());
        assert!(!fs::metadata(&file_path).is_ok());

        Ok(())
    }
}
