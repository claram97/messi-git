use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::logger::Logger;
use crate::utils::get_current_time;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Logs the 'git init' command with the specified Git directory.
///
/// This function logs the 'git init' command with the provided Git directory
/// to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `git_dir` - A `Path` representing the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_init(git_dir: &Path) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git init': Initialized repository at '{}'{}",
        git_dir.display(),
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// `create_directory_if_not_exists` is a utility function that creates a directory if it doesn't exist.
///
/// ## Parameters
///
/// - `directory`: Path to the directory to create.
///
/// ## Returns
///
/// Returns an `io::Result<()>` indicating whether the directory creation was successful or if an error occurred.
///
fn create_directory_if_not_exists(directory: &str) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    Ok(())
}

/// `create_file_if_not_exists` is a utility function that creates a file if it doesn't exist and writes the specified content to it.
///
/// ## Parameters
///
/// - `file`: Path to the file to create.
/// - `content`: Content to write to the file.
///
/// ## Returns
///
/// Returns an `io::Result<()>` indicating whether the file creation and write operation were successful or if an error occurred.
///
fn create_file_if_not_exists(file: &str, content: &str) -> io::Result<()> {
    if fs::metadata(file).is_err() {
        let mut file = fs::File::create(file)?;
        file.write_all(content.as_bytes())?;
    }
    Ok(())
}

/// `git_init` is a function that initializes a simulated Git repository in the specified directory.
///
/// ## Parameters
///
/// - `directory`: Path to the directory where the repository will be initialized.
/// - `initial_branch`: Name of the initial branch.
/// - `template_directory`: Optional path to a template directory to copy files from.
///
/// ## Returns
///
/// Returns an `io::Result<()>` indicating whether the operation was successful or if an error occurred.
///
pub fn git_init(
    directory: &str,
    git_dir_name: &str,
    initial_branch: &str,
    template_directory: Option<&str>,
) -> io::Result<()> {
    // Create directory if it doesn't exist
    if !Path::new(directory).exists() {
        fs::create_dir_all(directory)?;
    }

    // Necessary directories
    let git_dir = format!("{}/{}", directory, git_dir_name);
    create_directory_if_not_exists(&git_dir)?;

    create_directory_if_not_exists(&format!("{}/objects", &git_dir))?;
    create_directory_if_not_exists(&format!("{}/refs/heads", &git_dir))?;
    create_directory_if_not_exists(&format!("{}/refs/tags", &git_dir))?; // Create 'refs/tags' directory

    // HEAD file
    let head_content = format!("ref: refs/heads/{}\n", initial_branch);
    let head_file = format!("{}/HEAD", &git_dir);
    create_file_if_not_exists(&head_file, &head_content)?;

    // Config file
    let config_content = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";
    let config_file = format!("{}/config", &git_dir);
    create_file_if_not_exists(&config_file, config_content)?;

    // Index file
    let index_file = format!("{}/index", &git_dir);
    create_file_if_not_exists(&index_file, "")?;

    // Copy files from the template directory
    if let Some(template) = template_directory {
        let template_dir = Path::new(template);
        let repo_dir = Path::new(directory);
        for entry in fs::read_dir(template_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let src = entry.path();
            let destination = repo_dir.join(file_name);
            fs::copy(&src, &destination)?;
        }
    }

    println!(
        "Git repository initialized successfully in '{}'.",
        directory
    );
    log_init(Path::new(&git_dir))?;
    Ok(())
}

/// Creates a temporary directory and returns its path as a `Result`.
///
/// This function generates a temporary directory with a unique name in the system's
/// temporary directory. It attempts to create the directory and, if successful, returns
/// the path wrapped in `Ok`. If any step of the process fails, it returns an `Err` with
/// an error message.
///
/// # Returns
///
/// - `Ok(String)`: If the temporary directory is created successfully, contains the path
///   of the created directory as a string.
/// - `Err(String)`: If any step of the process fails, contains an error message.
///
///

#[cfg(test)]
mod test {

    use super::*;
    use crate::configuration::GIT_DIR_FOR_TEST;
    use crate::init::fs::File;
    use rand::random;
    use std::fs;
    use std::io::Read;

    fn create_temp_directory() -> Result<String, String> {
        let mut directory = std::env::temp_dir();
        directory.push(format!("git_init_test_{}", random::<u32>()));

        if let Err(err) = fs::create_dir(&directory) {
            return Err(format!("Failed to create temp directory: {}", err));
        }

        if let Some(path_str) = directory.to_str() {
            Ok(path_str.to_string())
        } else {
            Err("Failed to convert path to string".to_string())
        }
    }

    #[test]
    /// Tests the `create_directory_if_not_exists` function.
    fn test_create_directory_if_not_exists() {
        // Create a temporary directory
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        // Call the function to create the directory
        if let Err(err) = create_directory_if_not_exists(&temp_dir) {
            panic!("Failed to create directory: {}", err);
        }

        // Check if the directory exists
        assert!(Path::new(&temp_dir).exists());
        assert!(Path::new(&temp_dir).is_dir());

        // Clean up: Remove the temporary directory
        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }

    #[test]
    /// Tests the `create_file_if_not_exists` function.
    fn test_create_file_if_not_exists() {
        // Create a temporary directory
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        // Define a test file path within the temporary directory
        let file_path = format!("{}/test_file.txt", temp_dir);

        // Call the function to create the file with some content
        if let Err(err) = create_file_if_not_exists(&file_path, "Test content") {
            panic!("Failed to create file: {}", err);
        }

        // Check if the file exists and has the expected content
        assert!(Path::new(&file_path).exists());
        let mut file_content = String::new();
        if let Err(err) =
            File::open(&file_path).and_then(|mut file| file.read_to_string(&mut file_content))
        {
            panic!("Failed to read file: {}", err);
        }
        assert_eq!(file_content, "Test content");

        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }

    #[test]
    fn test_git_init() {
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        if let Err(err) = git_init(&temp_dir, GIT_DIR_FOR_TEST, "main", None) {
            panic!("Failed to initialize Git repository: {}", err);
        }

        let git_dir_path = format!("{}/{}", temp_dir, GIT_DIR_FOR_TEST);
        assert!(Path::new(&git_dir_path).exists());

        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }
    #[test]
    /// Test initializing a Git repository with the default branch "main".
    fn test_git_init_default_branch() {
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        match git_init(&temp_dir, GIT_DIR_FOR_TEST, "main", None) {
            Ok(_) => {
                let git_dir_path = format!("{}/{}", temp_dir, GIT_DIR_FOR_TEST);
                assert!(Path::new(&git_dir_path).exists());
                assert!(Path::new(&git_dir_path).is_dir());

                // Check the HEAD file
                let head_file_path = format!("{}/HEAD", git_dir_path);
                assert!(Path::new(&head_file_path).exists());

                let mut head_file_content = String::new();
                match File::open(&head_file_path)
                    .and_then(|mut file| file.read_to_string(&mut head_file_content))
                {
                    Ok(_) => {
                        assert_eq!(head_file_content, "ref: refs/heads/main\n");
                    }
                    Err(err) => panic!("Failed to read HEAD file: {}", err),
                }
            }
            Err(err) => panic!("Failed to initialize Git repository: {}", err),
        }

        // Clean up: Remove the temporary directory
        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }

    #[test]
    /// Test initializing a Git repository with a custom initial branch.
    fn test_git_init_custom_branch() {
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        match git_init(&temp_dir, GIT_DIR_FOR_TEST, "mybranch", None) {
            Ok(_) => {
                let git_dir_path = format!("{}/{}", temp_dir, GIT_DIR_FOR_TEST);
                assert!(Path::new(&git_dir_path).exists());
                assert!(Path::new(&git_dir_path).is_dir());

                // Check the HEAD file
                let head_file_path = format!("{}/HEAD", git_dir_path);
                assert!(Path::new(&head_file_path).exists());

                let mut head_file_content = String::new();
                match File::open(&head_file_path)
                    .and_then(|mut file| file.read_to_string(&mut head_file_content))
                {
                    Ok(_) => {
                        assert_eq!(head_file_content, "ref: refs/heads/mybranch\n");
                    }
                    Err(err) => panic!("Failed to read HEAD file: {}", err),
                }
            }
            Err(err) => panic!("Failed to initialize Git repository: {}", err),
        }

        // Clean up: Remove the temporary directory
        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }

    #[test]
    /// Test initializing a Git repository with a template directory.
    fn test_git_init_with_template() {
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        // Create a template directory with a sample file
        let template_dir = create_temp_directory().expect("Failed to create template directory");
        let template_file_path = format!("{}/template_file.txt", template_dir);
        create_file_if_not_exists(&template_file_path, "Template content")
            .expect("Failed to create template file");

        match git_init(&temp_dir, GIT_DIR_FOR_TEST, "main", Some(&template_dir)) {
            Ok(_) => {
                let git_dir_path = format!("{}/{}", temp_dir, GIT_DIR_FOR_TEST);
                assert!(Path::new(&git_dir_path).exists());
                assert!(Path::new(&git_dir_path).is_dir());

                // Check the HEAD file
                let head_file_path = format!("{}/HEAD", git_dir_path);
                assert!(Path::new(&head_file_path).exists());

                let mut head_file_content = String::new();
                match File::open(&head_file_path)
                    .and_then(|mut file| file.read_to_string(&mut head_file_content))
                {
                    Ok(_) => {
                        assert_eq!(head_file_content, "ref: refs/heads/main\n");
                    }
                    Err(err) => panic!("Failed to read HEAD file: {}", err),
                }

                // Check if the template file was copied
                let copied_template_file_path = format!("{}/template_file.txt", temp_dir);
                assert!(Path::new(&copied_template_file_path).exists());

                let mut template_file_content = String::new();
                match File::open(&copied_template_file_path)
                    .and_then(|mut file| file.read_to_string(&mut template_file_content))
                {
                    Ok(_) => {
                        assert_eq!(template_file_content, "Template content");
                    }
                    Err(err) => panic!("Failed to read copied template file: {}", err),
                }
            }
            Err(err) => panic!("Failed to initialize Git repository: {}", err),
        }

        // Clean up: Remove the temporary directories
        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
        if let Err(err) = fs::remove_dir_all(&template_dir) {
            panic!("Failed to remove template directory: {}", err);
        }
    }

    #[test]
    /// Test initializing a Git repository in an existing directory.
    fn test_git_init_existing_directory() {
        let temp_dir = create_temp_directory().expect("Failed to create temp directory");

        // Call git_init on an existing directory
        match git_init(&temp_dir, GIT_DIR_FOR_TEST, "main", None) {
            Ok(_) => {
                let git_dir_path = format!("{}/{}", temp_dir, GIT_DIR_FOR_TEST);
                assert!(Path::new(&git_dir_path).exists());
                assert!(Path::new(&git_dir_path).is_dir());
            }
            Err(err) => panic!("Failed to initialize Git repository: {}", err),
        }

        // Clean up: Remove the temporary directory
        if let Err(err) = fs::remove_dir_all(&temp_dir) {
            panic!("Failed to remove temp directory: {}", err);
        }
    }
}
