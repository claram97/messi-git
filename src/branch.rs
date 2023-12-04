use crate::commit;
use crate::commit::get_branch_name;
use crate::configuration::LOGGER_COMMANDS_FILE;
use crate::utils::get_current_time;
use crate::{logger::Logger, utils::obtain_git_dir};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

/// Returns the path inside the HEAD file.
/// The one that contains the path to the current branch.
/// If the file is empty, it returns an error.
pub fn get_current_branch_path(git_dir_path: &str) -> io::Result<String> {
    let head_path = git_dir_path.to_string() + "/HEAD";
    let mut head_file = std::fs::File::open(head_path)?;
    let mut head_content = String::new();
    head_file.read_to_string(&mut head_content)?;
    let path = match head_content.split(' ').last() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HEAD file is empty\n",
            ))
        }
    };
    let nombre: Vec<&str> = path.split('\n').collect();
    let path_final = nombre[0];
    Ok(path_final.to_string())
}

/// Retrieves the commit hash associated with a specific Git branch.
///
/// This function reads the commit hash associated with a given Git branch named `branch_name` from the local
/// Git repository located in the directory specified by `git_dir`. It accesses the branch reference file to
/// obtain the commit hash.
///
/// # Arguments
///
/// * `branch_name`: The name of the Git branch for which to retrieve the commit hash.
/// * `git_dir`: The path to the local directory containing the Git repository.
///
/// # Returns
///
/// Returns a `Result` containing the commit hash of the specified branch in case of success, or an error
/// in case any issue occurs during the operation.
///
pub fn get_branch_commit_hash(branch_name: &str, git_dir: &str) -> io::Result<String> {
    let branch_path = git_dir.to_string() + "/refs/heads/" + branch_name;
    let mut branch_file = std::fs::File::open(branch_path)?;
    let mut branch_content = String::new();
    branch_file.read_to_string(&mut branch_content)?;
    let nombre: Vec<&str> = branch_content.split('\n').collect();
    let path_final = nombre[0];
    Ok(path_final.to_string())
}

/// Updates the commit hash associated with a Git branch in the local repository.
///
/// This function allows you to update the commit hash associated with a specific Git branch named `branch_name`
/// in the local Git repository located in the directory specified by `git_dir`. It writes the provided `commit_hash`
/// to the branch's reference file, effectively changing the commit the branch points to.
///
/// # Arguments
///
/// * `branch_name`: The name of the Git branch to update.
/// * `commit_hash`: The new commit hash to associate with the branch.
/// * `git_dir`: The path to the local directory containing the Git repository.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
///
pub fn update_branch_commit_hash(
    branch_name: &str,
    commit_hash: &str,
    git_dir: &str,
) -> io::Result<()> {
    let branch_path = git_dir.to_string() + "/refs/heads/" + branch_name;
    let mut branch_file = std::fs::File::create(branch_path)?;
    branch_file.write_all(commit_hash.as_bytes())?;
    Ok(())
}

/// Retrieves the commit hash of the current branch in the local Git repository.
///
/// This function reads the commit hash of the currently checked-out branch in the local Git repository located
/// in the directory specified by `git_dir_path`. It accesses the reference file of the current branch to obtain
/// the commit hash.
///
/// # Arguments
///
/// * `git_dir_path`: The path to the local directory containing the Git repository.
///
/// # Returns
///
/// Returns a `Result` containing the commit hash of the current branch in case of success, or an error
/// in case any issue occurs during the operation.
///
pub fn get_current_branch_commit(git_dir_path: &str) -> io::Result<String> {
    let branch_path = get_current_branch_path(git_dir_path)?;
    let complete_path = git_dir_path.to_string() + "/" + &branch_path;
    let mut branch_file = File::open(complete_path)?;
    let mut branch_content = String::new();
    branch_file.read_to_string(&mut branch_content)?;
    Ok(branch_content)
}

/// Deletes a Git branch from the local repository.
///
/// This function is used to delete a specific Git branch named `branch_name` from the local Git repository
/// located in the directory specified by `git_dir`. If the branch exists, its reference file is removed.
/// If the branch does not exist, an error message is printed to the standard output.
///
/// # Arguments
///
/// * `git_dir`: The path to the local directory containing the Git repository.
/// * `branch_name`: The name of the branch to be deleted.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure. In case of success, an `io::Result<()>` is returned.
///
pub fn delete_branch(git_dir: &str, branch_name: &str, output: &mut impl Write) -> io::Result<()> {
    let branch_path = git_dir.to_string() + "/refs/heads/" + branch_name;
    let path = Path::new(&branch_path);

    if path.exists() {
        let path_relative_to_refs = format!("{}/{}", "refs/heads", branch_name);
        let head_path = format!("{}/HEAD", git_dir);
        let content = fs::read_to_string(head_path)?;
        if content.eq(&path_relative_to_refs) {
            if let Some(working_dir) = Path::new(git_dir).parent() {
                output.write_all(
                    format!(
                        "error: Cannot delete branch {} checked out at {}",
                        branch_name,
                        working_dir.to_string_lossy()
                    )
                    .as_bytes(),
                )?;
            } else {
                output.write_all("Error getting working dir.\n".as_bytes())?;
            }
        } else {
            let content = fs::read_to_string(&branch_path)?;
            let content = content.chars().take(7).collect::<String>();

            fs::remove_file(path)?;
            output.write_all(format!("Deleted {} (was {}\n)", branch_name, content).as_bytes())?;
        }
    } else {
        let buffer = format!("error: branch '{}' not found\n", branch_name);
        output.write_all(buffer.as_bytes())?;
    }

    Ok(())
}

/// Creates a new branch from an existing one in a Git repository.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git repository directory.
/// * `branch_name` - The name of the new branch to be created.
/// * `from` - The name of the existing branch to base the new branch on.
/// * `output` - A mutable reference to a type implementing the `Write` trait where output messages
///              will be written. This can be a file, standard output (`stdout`), etc.
///
/// # Errors
///
/// This function returns an `io::Result` indicating whether the operation was successful or
/// encountered an error.
///
fn create_branch_from_existing_one(
    git_dir: &str,
    branch_name: &str,
    from: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let new_refs = (&git_dir).to_string() + "/refs/heads/" + branch_name;
    let refs_path = Path::new(&new_refs);
    if refs_path.exists() {
        let buffer = format!("fatal: A branch named '{}' already exists\n", branch_name);
        output.write_all(buffer.as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("fatal: A branch named '{}' already exists\n", branch_name),
        ));
    }

    let from_refs = (&git_dir).to_string() + "/refs/heads/" + from;
    let from_path = Path::new(&from_refs);
    if !from_path.exists() {
        let buffer = format!("fatal: Not a valid object name: '{}'.\n", from);
        output.write_all(buffer.as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fatal: Not a valid object name: '{}'.\n", from),
        ));
    }

    let commit_hash = fs::read_to_string(from_path)?;
    let mut new_branch_file = File::create(refs_path)?;
    new_branch_file.write_all(commit_hash.as_bytes())?;
    new_branch_file.flush()?;
    Ok(())
}

/// Creates a new branch from the current branch in a Git repository.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git repository directory.
/// * `branch_name` - The name of the new branch to be created.
/// * `output` - A mutable reference to a type implementing the `Write` trait where output messages
///              will be written. This can be a file, standard output (`stdout`), etc.
///
/// # Errors
///
/// This function returns an `io::Result` indicating whether the operation was successful or
/// encountered an error.
///
fn create_branch_from_current_one(
    git_dir: &str,
    branch_name: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(heads_dir)?;
    if entries.count() == 0 {
        let buffer = "fatal: Please commit something to create a branch\n".to_string();
        output.write_all(buffer.as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fatal: Please commit something to create a branch\n".to_string(),
        ));
    }

    let new_refs = (&git_dir).to_string() + "/refs/heads/" + branch_name;
    let refs_path = Path::new(&new_refs);
    if refs_path.exists() {
        let buffer = format!("fatal: A branch named '{}' already exists\n", branch_name);
        output.write_all(buffer.as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fatal: A branch named '{}' already exists\n", branch_name),
        ));
    }
    let current_commit = get_current_branch_commit(git_dir)?;
    let mut file = File::create(&new_refs)?;
    file.write_all(current_commit.as_bytes())?;
    Ok(())
}

/// Creates a new branch in the repo with the given name.
/// The new branch will point to the same commit as the current branch.
/// HEAD won't be updated.
///
/// ## Arguments
/// * `git_dir` - The path to the repo directory.
/// * `branch_name` - The name of the new branch.
/// * `output` - The output to write the error message if any.
///
/// ## Errors
/// If the branch already exists, the branch is not created and an error is returned.
/// If the HEAD file is empty, an error is returned.
/// If there are no tracked files, an error is returned.
pub fn create_new_branch(
    git_dir: &str,
    branch_name: &str,
    from: Option<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    if let Some(from) = from {
        create_branch_from_existing_one(git_dir, branch_name, from, output)?;
    } else {
        create_branch_from_current_one(git_dir, branch_name, output)?;
    }
    Ok(())
}

/// Lists all the branches in the repo. It writes the output in the given output.
/// If the branch is the current one, it will be marked with a `*` and in green.
pub fn list_branches(git_dir: &str, output: &mut impl Write) -> io::Result<()> {
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(&heads_dir)?;
    let current_branch = commit::get_branch_name(git_dir)?;
    if entries.count() > 0 {
        let entries = fs::read_dir(&heads_dir)?;
        for entry in entries {
            let entry = entry?;
            if current_branch.eq(&entry.file_name().to_string_lossy().to_string()) {
                let buffer = format!("*\x1B[32m {}\x1B[0m\n", entry.file_name().to_string_lossy());
                output.write_all(buffer.as_bytes())?;
            } else {
                let buffer = format!("  {}\n", entry.file_name().to_string_lossy());
                output.write_all(buffer.as_bytes())?;
            }
        }
    }

    Ok(())
}

/// Modifies the name of an existing branch in a Git repository.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git repository directory.
/// * `branch_name` - The name of the branch to be modified.
/// * `new_name` - The new name for the branch.
/// * `output` - A mutable reference to a type implementing the `Write` trait where output messages
///              will be written. This can be a file, standard output (`stdout`), etc.
///
/// # Errors
///
/// This function returns an `io::Result` indicating whether the operation was successful or
/// encountered an error.
///
pub fn modify_branch(
    git_dir: &str,
    branch_name: &str,
    new_name: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let branch_path = PathBuf::from(git_dir).join("refs/heads").join(branch_name);
    let new_branch_path = PathBuf::from(git_dir).join("refs/heads").join(new_name);

    if branch_path.exists() {
        if new_branch_path.exists() {
            output.write_all(
                format!("fatal: A branch named {} already exists.\n", new_name).as_bytes(),
            )?;
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("fatal: A branch named {} already exists.\n", new_name),
            ));
        } else {
            let current_branch = get_branch_name(git_dir)?;
            if current_branch.eq(branch_name) {
                let head_path = format!("{}/HEAD", git_dir);
                let mut head_file = File::create(head_path)?;
                head_file.write_all(format!("ref: refs/heads/{}", new_name).as_bytes())?;
                head_file.flush()?;
            }
            fs::rename(&branch_path, &new_branch_path)?;
        }
    } else {
        let error_message = format!(
            "error: refname refs/heads/{} not found\nfatal: Branch rename failed\n",
            branch_name
        );
        output.write_all(error_message.as_bytes())?;
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, error_message));
    }

    Ok(())
}
/// Lists all the branches in the repo or creates a new branch depending on the argument.
///
/// ## Arguments
/// * `name` - The name of the new branch. If it's `None`, the current branches are listed.
///
/// ## Errors
/// If the branch already exists, the branch is not created and an error is returned.
/// If the HEAD file is empty, an error is returned.
/// If there are no tracked files, an error is returned.
/// If the git directory is not found, an error is returned.
pub fn git_branch(
    name: Option<String>,
    option: Option<&str>,
    new_name: Option<&str>,
    output: &mut impl Write,
) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    if let Some(name) = name {
        if let Some(option) = option {
            match option {
                "-l" => list_branches(&git_dir, output)?,
                "-d" => delete_branch(&git_dir, &name, output)?,
                "-m" => {
                    if let Some(new_name) = new_name {
                        modify_branch(&git_dir, &name, new_name, output)?;
                    }
                }
                "-c" => {
                    create_new_branch(&git_dir, &name, None, output)?;
                }
                _ => {
                    output.write_all(b"fatal: Invalid option.\n")?;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid option\n",
                    ));
                }
            }
        } else if let Some(new_name) = new_name {
            create_new_branch(&git_dir, &name, Some(new_name), output)?;
        } else {
            create_new_branch(&git_dir, &name, None, output)?;
        }
    } else if let Some(new_name) = new_name {
        if name.is_none() {
            let current_branch = get_branch_name(&git_dir)?;
            modify_branch(&git_dir, &current_branch, new_name, output)?;
        }
    } else {
        list_branches(&git_dir, output)?;
    }
    log_command(
        "git_branch",
        option.unwrap_or_default(),
        &PathBuf::from(&git_dir),
    )?;
    Ok(())
}

/// Logs a custom Git command with the specified parameters.
///
/// This function logs a custom Git command with the provided parameters to a file named
/// 'logger_commands.txt'.
///
/// # Arguments
///
/// * `command` - A string slice representing the Git command.
/// * `option` - A string slice representing the command option or additional information.
/// * `git_dir` - A `Path` representing the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
fn log_command(command: &str, option: &str, _git_dir: &Path) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!("Command '{}': {} {}", command, option, get_current_time());
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// Returns a vector with the names of all the branches in the repo.
///
/// ## Arguments
/// * `git_dir` - The path to the repo directory.
///
/// ## Errors
/// If the git directory is not found, an error is returned.
/// If the branches directory is not found, an error is returned.
pub fn get_all_branches(git_dir: &str) -> io::Result<Vec<String>> {
    let mut branches = vec![];
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(&heads_dir)?;
    if entries.count() > 0 {
        let entries = fs::read_dir(&heads_dir)?;
        for entry in entries {
            let entry = entry?;
            branches.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(branches)
}

/// Removes ANSI escape codes from the input string.
///
/// This function takes an input string and removes ANSI escape codes used for color formatting.
///
/// # Arguments
///
/// * `input` - The input string with ANSI escape codes.
///
/// # Returns
///
/// A new string with the ANSI escape codes removed.
pub fn remove_ansi_escape_codes(input: &str) -> String {
    let mut output = String::new();
    let mut in_escape = false;

    for c in input.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1B' {
            in_escape = true;
        } else {
            output.push(c);
        }
    }

    output
}

/// Checks if a Git branch exists in the specified Git directory.
///
/// This function determines the existence of a Git branch by checking if the corresponding
/// branch file exists in the "refs/heads/" directory of the Git repository.
///
/// # Arguments
///
/// - `branch`: The name of the Git branch to check for existence.
/// - `git_dir`: The path to the Git directory where the repository is located.
///
/// # Returns
///
/// - `true`: The specified Git branch exists in the provided Git directory.
/// - `false`: The specified Git branch does not exist in the provided Git directory.
///
pub fn is_an_existing_branch(branch: &str, git_dir: &str) -> bool {
    let path = format!("{}/refs/heads/{}", git_dir, branch);

    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file()
    } else {
        false
    }
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
    fn test_remove_ansi_escape_codes() {
        let input = "\x1B[32mThis is green text\x1B[0m";
        let expected_output = "This is green text";
        let output = remove_ansi_escape_codes(input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_remove_ansi_escape_codes_no_escape_codes() {
        let input = "This is plain text";
        let expected_output = "This is plain text";
        let output = remove_ansi_escape_codes(input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_list_branches() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches", true)?;
        init::git_init(
            "tests/test_list_branches",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_1", false)?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_2", false)?;
        create_if_not_exists(
            "tests/test_list_branches/.mgit/refs/heads/current_branch",
            false,
        )?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_3", false)?;
        let mut output: Vec<u8> = vec![];
        list_branches("tests/test_list_branches/.mgit", &mut output)?;
        assert!(!output.is_empty());
        std::fs::remove_dir_all("tests/test_list_branches")?;
        Ok(())
    }

    #[test]
    fn test_list_branches_empty() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_2", true)?;
        init::git_init(
            "tests/test_list_branches_2",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        let mut output: Vec<u8> = vec![];
        list_branches("tests/test_list_branches_2/.mgit", &mut output)?;
        assert!(output.is_empty());
        std::fs::remove_dir_all("tests/test_list_branches_2")?;
        Ok(())
    }

    #[test]
    fn test_modify_current_branch_correctly() -> io::Result<()> {
        let path = "tests/branch_test_modify_repo";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let current_branch_string_path = format!("{}/{}", git_dir, "refs/heads/current_branch");
        let _current_branch_file = File::create(&current_branch_string_path)?;
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let current_brach_path = Path::new(&current_branch_string_path);
        let new_branch_path = Path::new(&new_branch_string_path);
        let head_path = format!("{}/{}", git_dir, "HEAD");
        let heads_content = fs::read_to_string(&head_path)?;
        let before = (current_brach_path.exists())
            & (!new_branch_path.exists())
            & heads_content.starts_with("ref: refs/heads/current_branch");
        let new_branch_name = "branch";
        let mut output: Vec<u8> = vec![];
        modify_branch(&git_dir, "current_branch", new_branch_name, &mut output)?;
        let heads_content = fs::read_to_string(&head_path)?;
        let after = (!current_brach_path.exists())
            & (new_branch_path.exists())
            & heads_content.starts_with("ref: refs/heads/branch");
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_modify_not_current_branch_correctly() -> io::Result<()> {
        let path = "tests/branch_test_modify_repo_2";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let branch_to_modify_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let _branch_to_modify_file = File::create(&branch_to_modify_string_path)?;
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/new_branch");
        let branch_to_modify_path = Path::new(&branch_to_modify_string_path);
        let new_branch_path = Path::new(&new_branch_string_path);
        let head_path = format!("{}/{}", git_dir, "HEAD");
        let heads_content = fs::read_to_string(&head_path)?;
        let before = (branch_to_modify_path.exists())
            & (!new_branch_path.exists())
            & heads_content.starts_with("ref: refs/heads/current_branch");
        let new_branch_name = "new_branch";
        let mut output: Vec<u8> = vec![];
        modify_branch(&git_dir, "branch", new_branch_name, &mut output)?;
        let heads_content = fs::read_to_string(&head_path)?;
        let after = (!branch_to_modify_path.exists())
            & (new_branch_path.exists())
            & heads_content.starts_with("ref: refs/heads/current_branch");
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_modify_non_existing_branch_informs_error() -> io::Result<()> {
        let path = "tests/branch_test_modify_repo_3";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let non_existing_branch_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/new_branch");
        let non_existing_branch_path = Path::new(&non_existing_branch_string_path);
        let new_branch_path = Path::new(&new_branch_string_path);
        let before = (!non_existing_branch_path.exists()) & (!new_branch_path.exists());
        let new_branch_name = "new_branch";
        let mut output: Vec<u8> = vec![];
        let result = modify_branch(&git_dir, "branch", new_branch_name, &mut output);
        assert!(result.is_err());
        let after = (!non_existing_branch_path.exists()) & (!new_branch_path.exists());
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_modify_branch_to_an_existing_one_informs_error() -> io::Result<()> {
        let path = "tests/branch_test_modify_repo_4";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let existing_branch_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let _existing_branch_file = File::create(&existing_branch_string_path)?;
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/new_branch");
        let _new_branch_file = File::create(&new_branch_string_path)?;
        let existing_branch_path = Path::new(&existing_branch_string_path);
        let new_branch_path = Path::new(&new_branch_string_path);
        let before = (existing_branch_path.exists()) & (new_branch_path.exists());
        let new_branch_name = "new_branch";
        let mut output: Vec<u8> = vec![];
        let result = modify_branch(&git_dir, "branch", new_branch_name, &mut output);
        assert!(result.is_err());
        let after = (existing_branch_path.exists()) & (new_branch_path.exists());
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_branch_from_non_existing_one_informs_error() -> io::Result<()> {
        let path = "tests/branch_test_create_repo_0";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let current_branch_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let current_branch_path = Path::new(&current_branch_string_path);
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/new_branch");
        let new_branch_path = Path::new(&new_branch_string_path);
        let before = !current_branch_path.exists() & !new_branch_path.exists();
        let mut output: Vec<u8> = vec![];
        let _result = create_new_branch(&git_dir, "new_branch", Some("branch"), &mut output);
        let after = !current_branch_path.exists() & !new_branch_path.exists();
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_branch_from_existing_one_functions_correctly() -> io::Result<()> {
        let path = "tests/branch_test_create_repo_";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let current_branch_string_path = format!("{}/{}", git_dir, "refs/heads/branch");
        let _current_branch_file = File::create(&current_branch_string_path)?;
        let current_branch_path = Path::new(&current_branch_string_path);
        let new_branch_string_path = format!("{}/{}", git_dir, "refs/heads/new_branch");
        let new_branch_path = Path::new(&new_branch_string_path);
        let before = current_branch_path.exists() & !new_branch_path.exists();
        let mut output: Vec<u8> = vec![];
        let _result = create_new_branch(&git_dir, "new_branch", Some("branch"), &mut output);
        let after = current_branch_path.exists() & new_branch_path.exists();
        assert!(before & after);
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_delete_branch_correctly() -> io::Result<()> {
        let path = "tests/branch_test_delete_repo";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let branch_to_delete_path = format!("{}/{}", git_dir, "/refs/heads/branch");
        File::create(&branch_to_delete_path)?;
        let branch_to_delete_path = Path::new(&branch_to_delete_path);
        let before = branch_to_delete_path.exists();
        // let mut current_branch_file = File::create(current_branch_path)?;
        // current_branch_file.write_all("12345678910".as_bytes())?;
        let mut output: Vec<u8> = vec![];
        delete_branch(&git_dir, "branch", &mut output)?;
        let after = branch_to_delete_path.exists();
        assert!((before == true) && (after == false));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_deleting_non_existing_branch_prints_error() -> io::Result<()> {
        let path = "tests/branch_test_delete_repo_2";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let mut output: Vec<u8> = vec![];
        delete_branch(&git_dir, "branch", &mut output)?;
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("error"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_deleting_checkouted_branch_cant_be_performed() -> io::Result<()> {
        let path = "tests/branch_test_delete_repo_3";
        let git_dir = format!("{}/{}", path, ".mgit");
        init::git_init(path, GIT_DIR_FOR_TEST, "current_branch", None)?;
        let mut output: Vec<u8> = vec![];
        delete_branch(&git_dir, "current_branch", &mut output)?;
        let string = String::from_utf8(output).unwrap();
        assert!(string.contains("error"));
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn test_create_new_branch_already_exists() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_3", true)?;
        init::git_init(
            "tests/test_list_branches_3",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists(
            "tests/test_list_branches_3/.mgit/refs/heads/current_branch",
            false,
        )?;
        let mut output: Vec<u8> = vec![];
        let result = create_new_branch(
            "tests/test_list_branches_3/.mgit",
            "current_branch",
            None,
            &mut output,
        );
        assert!(result.is_err());
        assert!(!output.is_empty());
        let result = String::from_utf8(output);
        if result.is_ok() {
            let string = result.unwrap();
            assert!(string.starts_with("fatal: A branch named 'current_branch' already exists\n"));
        }

        std::fs::remove_dir_all("tests/test_list_branches_3")?;
        Ok(())
    }

    #[test]
    fn test_create_new_branch() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_4", true)?;
        init::git_init(
            "tests/test_list_branches_4",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        create_if_not_exists(
            "tests/test_list_branches_4/.mgit/refs/heads/current_branch",
            false,
        )?;

        let mut current_branch_file =
            File::create("tests/test_list_branches_4/.mgit/refs/heads/current_branch")?;
        let commit_hash = "aaaaaaaaaaaaaaaaaaaaaa";
        current_branch_file.write_all(commit_hash.as_bytes())?;

        let mut output: Vec<u8> = vec![];
        create_new_branch(
            "tests/test_list_branches_4/.mgit",
            "my_branch",
            None,
            &mut output,
        )?;

        let mut head_file = std::fs::File::open("tests/test_list_branches_4/.mgit/HEAD")?;
        let mut head_content = String::new();
        head_file.read_to_string(&mut head_content)?;

        let mut new_branch_file =
            std::fs::File::open("tests/test_list_branches_4/.mgit/refs/heads/my_branch")?;
        let mut new_branch_content = String::new();
        new_branch_file.read_to_string(&mut new_branch_content)?;

        assert_eq!(output.len(), 0); //No output means ok.
        assert_eq!(head_content, "ref: refs/heads/current_branch\n");
        assert_eq!(new_branch_content, commit_hash);

        std::fs::remove_dir_all("tests/test_list_branches_4")?;
        Ok(())
    }

    #[test]
    fn test_create_new_branch_with_no_tracked_files() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_5", true)?;
        init::git_init(
            "tests/test_list_branches_5",
            GIT_DIR_FOR_TEST,
            "current_branch",
            None,
        )?;
        let mut output: Vec<u8> = vec![];
        let result = create_new_branch(
            "tests/test_list_branches_5/.mgit",
            "my_branch",
            None,
            &mut output,
        );
        assert!(result.is_err());
        assert!(!output.is_empty());
        let result = String::from_utf8(output);
        if result.is_ok() {
            let string = result.unwrap();
            assert!(string.starts_with("fatal: Please commit something to create a branch\n"));
        }
        std::fs::remove_dir_all("tests/test_list_branches_5")?;
        Ok(())
    }
}
