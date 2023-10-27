use crate::branch;
use crate::tree_handler;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

/// Process command-line arguments and options to perform various actions in a Git-like application.
///
/// This function expects command-line arguments and options in the form of `<option> <branch_or_commit>`.
///
/// # Arguments
///
/// * `option` - A string representing the option to be performed. Possible values are:
///   - `""`: Change to the specified branch.
///   - `"-b"`: Create and change to a new branch.
///   - `"-B"`: Create or reset a branch if it exists.
///   - `"--detach"`: Change to a specific commit (detached mode).
///   - `"-f"`: Force the change of branch or commit (discarding uncommitted changes).
///
/// * `destination` - A string representing the branch name or commit ID to be operated on.
pub fn process_args(git_dir_path: &str, root_dir: &str) -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: my_git_checkout <option> <branch_or_commit>");
        std::process::exit(1);
    }

    let option = &args[1];
    let destination = &args[2];
    let git_dir = Path::new(git_dir_path);

    match option.as_str() {
        // Change to the specified branch
        "" => checkout_branch(git_dir, root_dir, destination),
        // Create and change to a new branch
        "-b" => create_and_checkout_branch(git_dir, root_dir, destination),
        // Create or reset a branch if it exists
        "-B" => create_or_reset_branch(git_dir, root_dir, destination),
        // Change to a specific commit (detached mode)
        "--detach" => checkout_commit_detached(git_dir, root_dir, destination),
        // Force the change of branch or commit (discarding uncommitted changes)
        "-f" => {
            force_checkout(git_dir, destination);
            Ok(())
        }
        _ => {
            eprintln!("Invalid option: {}", option);
            std::process::exit(1);
        }
    }
}

/// Checkout a specific branch by updating the HEAD reference in a Git-like repository.
///
/// This function is responsible for changing the currently checked-out branch in the repository.
///
/// # Arguments
///
/// * `git_dir` - A reference to the root directory of the Git repository, represented as a `Path`.
/// * `branch_name` - A string representing the name of the branch to be checked out.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use messi::checkout::checkout_branch;
/// let git_dir = Path::new(".mgit");
/// let branch_name = "my_branch";
/// let root_dir = ".";
/// checkout_branch(&git_dir, root_dir, branch_name);
/// ```
///
/// This function checks if the specified branch reference file exists. If it exists, the content of
/// the reference file is read to determine the commit it points to. Then, it updates the HEAD file
/// to point to the new branch, effectively switching to that branch.
///
/// If the branch reference file does not exist, or if there are errors during the process, the
/// function prints an error message to the standard error output.
pub fn checkout_branch(git_dir_path: &Path, root_dir: &str, branch_name: &str) -> io::Result<()> {
    let git_dir_path_str = match git_dir_path.to_str() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error when reading path",
            ))
        }
    };

    match checkout_branch_references(git_dir_path, branch_name) {
        Ok(old_commit_id) => {
            let new_commit_id = branch::get_current_branch_commit(git_dir_path_str)?;
            match replace_working_tree(git_dir_path_str, root_dir, &old_commit_id, &new_commit_id) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

fn checkout_branch_references(git_dir: &Path, branch_name: &str) -> io::Result<String> {
    let refs_dir = git_dir.join("refs").join("heads");
    let branch_ref_file = refs_dir.join(branch_name);
    let git_dir_str = match git_dir.to_str() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error when reading path",
            ))
        }
    };
    if !branch_ref_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Branch {} does not exist", branch_name),
        ));
    }
    let old_commit_id = branch::get_current_branch_commit(git_dir_str)?;
    let head_file = git_dir.join("HEAD");
    let new_head_content = format!("ref: refs/heads/{}\n", branch_name);
    fs::write(head_file, new_head_content)?;
    Ok(old_commit_id)
}

/// Create and checkout a new branch in a Git-like repository.
///
/// This function creates a new branch in the repository and sets it as the currently
/// checked-out branch. If the branch already exists, it provides an error message and
/// suggests using the `-B` option to reset it.
///
/// # Arguments
///
/// * `git_dir` - A reference to the root directory of the Git repository, represented as a `Path`.
/// * `branch_name` - A string representing the name of the new branch to be created and checked out.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use messi::checkout::create_and_checkout_branch;
/// let git_dir = Path::new(".mgit");
/// let branch_name = "my_new_branch";
/// let root_dir = ".";
/// create_and_checkout_branch(&git_dir, root_dir, branch_name);
/// ```
///
/// This function checks if the branch already exists in the repository. If it does, it prints an
/// error message and advises using the `-B` option to reset the branch. If the branch does not
/// exist, it creates a reference file for the new branch and writes an initial reference value
/// (which can be the ID of an initial commit). It then updates the HEAD file to point to the new
/// branch, effectively switching to the newly created branch.
///
/// If there are any errors during the branch creation process, the function prints appropriate
/// error messages to the standard error output.
pub fn create_and_checkout_branch(
    git_dir: &Path,
    root_dir: &str,
    branch_name: &str,
) -> io::Result<()> {
    let git_dir_str = match git_dir.to_str() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error when reading path",
            ))
        }
    };

    let old_commit_id = create_and_checkout_branch_references(git_dir_str, branch_name)?;
    let branch_commit_id = branch::get_current_branch_commit(git_dir_str)?;
    replace_working_tree(git_dir_str, root_dir, &old_commit_id, &branch_commit_id)?;
    Ok(())
}

fn create_and_checkout_branch_references(
    git_dir_str: &str,
    branch_name: &str,
) -> io::Result<String> {
    branch::create_new_branch(git_dir_str, branch_name, &mut io::stdout())?;
    let old_commit_id = checkout_branch_references(Path::new(git_dir_str), branch_name)?;
    Ok(old_commit_id)
}

/// Write a reference value to a file in a Git-like repository.
///
/// This function writes a reference value, typically a commit ID, to a specified file within
/// a Git-like repository. The reference file is represented by a mutable reference to a
/// `fs::File` and the value to be written is provided as a string.
///
/// # Arguments
///
/// * `file` - A mutable reference to a `fs::File` that represents the reference file to write the value to.
/// * `value` - A string containing the reference value, typically a commit ID, to be written to the file.
///
/// # Returns
///
/// This function returns an `io::Result` indicating whether the write operation was successful. If
/// the write operation succeeds, `Ok(())` is returned. If any errors occur during the write operation,
/// an `Err` variant containing an error description is returned.
///
/// # Example
///
/// ```
///  use std::fs::File;
/// use messi::checkout::write_reference_value;
/// let mut file = File::create("my_reference.txt").expect("Failed to create file");
/// let value = "my_commit_id";
/// let result = write_reference_value(&mut file, value);
/// assert!(result.is_ok());
/// ```
///
/// This example demonstrates how to use the `write_reference_value` function to write a reference
/// value to a file. It creates a new file, `my_reference.txt`, writes the value "my_commit_id" to
/// the file, and checks if the write operation was successful.
pub fn write_reference_value(file: &mut fs::File, value: &str) -> io::Result<()> {
    // Write the value to the reference file
    file.write_all(value.as_bytes())?;
    Ok(())
}

/// Create or reset a Git-like branch in a repository.
///
/// This function is used to create a new branch or reset an existing branch within a Git-like
/// repository. It takes the path to the Git repository directory and the name of the branch as
/// arguments. If the branch does not exist, it is created. If the branch already exists, it is
/// reset, and the HEAD reference is updated to point to the branch.
///
/// # Arguments
///
/// * `git_dir` - A reference to the `std::path::Path` representing the Git repository directory.
/// * `branch_name` - A string containing the name of the branch to create or reset.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use messi::checkout::create_or_reset_branch;
/// let git_dir = Path::new(".mgit");
/// let branch_name = "my_branch";
/// let root_dir = ".";
/// create_or_reset_branch(&git_dir, root_dir, branch_name);
/// ```
///
/// This example demonstrates how to use the `create_or_reset_branch` function to create or reset a branch
/// named "my_branch" in a Git-like repository. If the branch already exists, it will be reset, and the
/// HEAD reference will be updated to point to the branch.
pub fn create_or_reset_branch(git_dir: &Path, root_dir: &str, branch_name: &str) -> io::Result<()> {
    let refs_dir = git_dir.join("refs").join("heads");
    let branch_ref_file = refs_dir.join(branch_name);
    let git_dir_str = match git_dir.to_str() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error when reading path",
            ))
        }
    };
    //Check if the branch reference file exists
    if branch_ref_file.exists() {
        branch::delete_branch(git_dir_str, branch_name)?;
        create_and_checkout_branch(git_dir, root_dir, branch_name)?;
    } else {
        create_and_checkout_branch(git_dir, root_dir, branch_name)?;
    }
    Ok(())
}

/// Check out a specific commit in detached mode in a Git-like repository.
///
/// This function allows you to switch to a specific commit in detached mode within a Git-like
/// repository. It takes the path to the Git repository directory and the commit ID as arguments.
/// If the specified commit exists in the repository, it updates the HEAD reference to point to
/// the commit in detached mode.
///
/// # Arguments
///
/// * `git_dir` - A reference to the `std::path::Path` representing the Git repository directory.
/// * `commit_id` - A string containing the ID of the commit to check out in detached mode.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use messi::checkout::checkout_commit_detached;
/// let git_dir = Path::new(".mgit");
/// let commit_id = "a1b2c3d4e5"; // Replace with an actual commit ID.
/// let root_dir = ".";
/// checkout_commit_detached(&git_dir, root_dir, commit_id);
/// ```
///
/// This example demonstrates how to use the `checkout_commit_detached` function to switch to a specific
/// commit in detached mode within a Git-like repository. Make sure to replace `"a1b2c3d4e5"` with the
/// actual commit ID you want to check out.
pub fn checkout_commit_detached(git_dir: &Path, root_dir: &str, commit_id: &str) -> io::Result<()> {
    let git_dir_str = match git_dir.to_str() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error when reading path",
            ))
        }
    };
    match checkout_commit_detached_references(git_dir_str, commit_id) {
        Ok(old_commit_id) => {
            match replace_working_tree(git_dir_str, root_dir, &old_commit_id, commit_id) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

fn checkout_commit_detached_references(git_dir_str: &str, commit_id: &str) -> io::Result<String> {
    let head_file = git_dir_str.to_string() + "/HEAD";
    let old_commit_id = branch::get_current_branch_commit(git_dir_str)?;
    let new_head_content = format!("{} (commit)\n", commit_id);
    fs::write(head_file, new_head_content)?;
    Ok(old_commit_id)
}

fn replace_working_tree(
    git_dir: &str,
    root_dir: &str,
    old_commit_id: &str,
    new_commit_id: &str,
) -> io::Result<()> {
    let commit_tree = tree_handler::load_tree_from_commit(new_commit_id, git_dir)?;
    let latest_tree = tree_handler::load_tree_from_commit(old_commit_id, git_dir)?;
    // let file_dir: &Path = match Path::new(git_dir).parent() {
    //     Some(path) => path,
    //     None => return Err(io::Error::new(io::ErrorKind::Other, "Error when reading parent dir")),
    // };

    latest_tree.delete_directories2(Path::new(root_dir))?;
    commit_tree.create_directories(root_dir, git_dir)?;
    Ok(())
}

/// Forcefully switch to a specific branch or commit in a Git-like repository.
///
/// This function allows you to forcibly switch to a specific branch or commit in a Git-like
/// repository. It takes the path to the Git repository directory and the name of the branch or
/// the commit ID as arguments. If a branch is specified, it updates the HEAD reference to point
/// to the branch. If a commit ID is specified, it updates the HEAD reference to point to the
/// commit in detached mode.
///
/// # Arguments
///
/// * `git_dir` - A reference to the `std::path::Path` representing the Git repository directory.
/// * `branch_or_commit` - A string containing the branch name (e.g., "my_branch") or the commit
///                       ID (e.g., "a1b2c3d4e5").
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use messi::checkout::force_checkout;
/// let git_dir = Path::new(".mgit");
/// let branch_or_commit = "my_branch"; // Replace with a branch name or commit ID.
/// force_checkout(&git_dir, branch_or_commit);
/// ```
///
/// This example demonstrates how to use the `force_checkout` function to forcibly switch to a
/// specific branch or commit within a Git-like repository. You can replace `"my_branch"` with
/// the actual branch name or commit ID you want to switch to.
pub fn force_checkout(git_dir: &Path, branch_or_commit: &str) {
    // Check if a branch or a commit is provided
    let is_branch = branch_or_commit.starts_with("refs/heads/");

    if is_branch {
        // Check if the specified branch exists
        let branch_name = branch_or_commit.trim_start_matches("refs/heads/");
        let refs_dir = git_dir.join("refs").join("heads");
        let branch_ref_file = refs_dir.join(branch_name);

        if branch_ref_file.exists() {
            // Update the HEAD file to force the branch change
            let head_file = git_dir.join("HEAD");
            let new_head_content = format!("ref: {}\n", branch_or_commit);
            if let Err(err) = fs::write(head_file, new_head_content) {
                eprintln!("Failed to update HEAD file: {}", err);
                return;
            }

            println!("Force switched to branch: {}", branch_name);
        } else {
            eprintln!("Branch '{}' not found in the repository", branch_name);
        }
    } else {
        // Check if the specified commit exists
        let objects_dir = git_dir.join("objects");
        let commit_id = branch_or_commit;

        if objects_dir.join(commit_id).exists() {
            // Update the HEAD file to force the commit change in "detached" mode
            let head_file = git_dir.join("HEAD");
            let new_head_content = format!("{} (commit)\n", commit_id);
            if let Err(err) = fs::write(head_file, new_head_content) {
                eprintln!("Failed to update HEAD file: {}", err);
                return;
            }

            println!("Force switched to commit (detached mode): {}", commit_id);
        } else {
            eprintln!(
                "Branch or commit '{}' not found in the repository",
                branch_or_commit
            );
        }
    }
}

// Importa las bibliotecas necesarias para los tests
#[cfg(test)]
mod tests {
    use crate::{add, commit};

    use super::*;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    // Define a test directory for the Git repository
    // const TEST_GIT_DIR: &str = "/tmp/test_git_repository";
    const TEST_GIT: &str = "/tmp/test_git";
    const TEST_CHECKOUT: &str = "/tmp/test_checkout";
    const TEST_CHECKOUT2: &str = "/tmp/test_checkout2";
    const T: &str = "/tmp/te";

    /// Unit test for the `checkout_branch` function.
    ///
    /// This test validates the behavior of the `checkout_branch` function. It does the following:
    /// 1. Creates a test directory for a Git-like repository if it does not exist.
    /// 2. Creates an example branch and sets the HEAD file accordingly.
    /// 3. Calls the `checkout_branch` function to switch to the specified branch.
    /// 4. Verifies that the HEAD file has been updated correctly to point to the new branch.
    ///
    /// This test ensures that the `checkout_branch` function correctly switches to the specified
    /// branch and updates the HEAD reference.
    #[test]
    fn test_force_checkout_branch_references() {
        // Create a test directory if it doesn't exist
        if !Path::new(TEST_CHECKOUT).exists() {
            fs::create_dir_all(TEST_CHECKOUT).expect("Failed to create test directory");
        }
        // Create a sample branch and set the HEAD file
        let refs_dir = Path::new(TEST_CHECKOUT).join("refs").join("heads");
        let branch_name = "my_branch";
        let branch_ref_file = refs_dir.join(branch_name);
        fs::create_dir_all(&branch_ref_file.parent().unwrap()).expect("Failed to create dirs");
        fs::write(&branch_ref_file, "commit_id").expect("Failed to write branch reference");

        let head_file = Path::new(TEST_CHECKOUT).join("HEAD");
        fs::write(&head_file, format!("ref: refs/heads/{}", branch_name))
            .expect("Failed to write HEAD file");

        // Execute the checkout_branch function
        let result = checkout_branch_references(Path::new(TEST_CHECKOUT), branch_name);

        // Verify that the HEAD file has been updated correctly
        let head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert!(result.is_ok());
        assert_eq!(head_contents, format!("ref: refs/heads/{}\n", branch_name));
    }

    /// Unit test for the `create_and_checkout_branch` function.
    ///
    /// This test validates the behavior of the `create_and_checkout_branch` function. It does the following:
    /// 1. Creates a test directory for a Git-like repository if it does not exist.
    /// 2. Calls the `create_and_checkout_branch` function to create and switch to a new branch named "new_branch."
    /// 3. Verifies that a new branch file has been created in the repository.
    /// 4. Verifies that the HEAD file has been updated to point to the new branch.
    ///
    /// This test ensures that the `create_and_checkout_branch` function correctly creates a new branch and
    /// updates the HEAD reference.
    #[test]
    fn test_create_and_checkout_branch_references() {
        // Create a test directory if it doesn't exist
        if !Path::new(TEST_CHECKOUT2).exists() {
            fs::create_dir_all(TEST_CHECKOUT2).expect("Failed to create test directory");
        }

        //Create a refs heads directory and write the old branch name
        let refs_dir = Path::new(TEST_CHECKOUT2).join("refs").join("heads");
        fs::create_dir_all(&refs_dir).expect("Failed to create dirs");
        let branch_ref_file = refs_dir.join("my_branch");
        fs::File::create(&branch_ref_file).expect("Failed to create file");

        //Create a HEAD file and write the old branch name
        let head_file = Path::new(TEST_CHECKOUT2).join("HEAD");
        fs::File::create(&head_file).expect("Failed to create file");
        let head_contents = format!("ref: refs/heads/my_branch\n");
        fs::write(&head_file, head_contents).expect("Failed to write HEAD file");

        //Write a commit hash to the old branch
        let mut old_branch_file = fs::File::create(branch_ref_file).unwrap();
        write_reference_value(&mut old_branch_file, "commit_id").unwrap();

        // Execute the create_and_checkout_branch function
        create_and_checkout_branch_references(TEST_CHECKOUT2, "new_branch").unwrap();

        // Verify that a new branch has been created
        let refs_dir = Path::new(TEST_CHECKOUT2).join("refs").join("heads");
        let branch_ref_file = refs_dir.join("new_branch");
        assert!(branch_ref_file.exists(), "Branch file not created");

        // Verify that the HEAD file has been updated
        let head_file = Path::new(TEST_CHECKOUT2).join("HEAD");
        let head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(head_contents, "ref: refs/heads/new_branch\n");
    }

    /// Unit test for the `force_checkout` function.
    ///
    /// This test validates the behavior of the `force_checkout` function for switching between branches
    /// and commits in detached mode. It does the following:
    ///
    /// 1. Creates a test directory for a Git-like repository if it does not exist.
    /// 2. Creates a sample branch and sets the HEAD file to point to another branch ("other_branch").
    /// 3. Calls the `force_checkout` function with an existing branch name, which should force
    ///    the change of the branch by updating the HEAD reference.
    /// 4. Verifies that the HEAD reference is correctly updated to point to the specified branch.
    /// 5. Calls the `force_checkout` function with a commit ID in detached mode.
    /// 6. Verifies that the HEAD reference is correctly updated to represent a detached commit.
    ///
    /// This test ensures that the `force_checkout` function correctly handles branch switching and detached commits
    /// by forcing the change of the HEAD reference.
    #[test]
    fn test_force_checkout_branch() {
        // Create a test directory if it does not exist
        if !Path::new(TEST_GIT).exists() {
            fs::create_dir_all(TEST_GIT).expect("Failed to create test directory");
        }

        // Create a sample branch and set the HEAD file
        let refs_dir = Path::new(TEST_GIT).join("refs").join("heads");
        let branch_name = "other_branch";
        let branch_ref_file = refs_dir.join(branch_name);
        fs::create_dir_all(&branch_ref_file.parent().unwrap()).expect("Failed to create dirs");
        fs::write(&branch_ref_file, "commit_id").expect("Failed to write branch reference");

        let head_file = Path::new(TEST_GIT).join("HEAD");
        fs::write(&head_file, format!("ref: refs/heads/other_branch\n"))
            .expect("Failed to write HEAD file");

        // Execute the force_checkout function with an existing branch
        force_checkout(Path::new(TEST_GIT), branch_name);

        // Verify that the HEAD file has been updated to force the branch change
        let head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(head_contents, format!("ref: refs/heads/{}\n", branch_name));
    }

    /// Unit test for the `checkout_commit_detached` function.
    ///
    /// This test validates the behavior of the `checkout_commit_detached` function when changing to
    /// a specific commit in detached mode. It follows these steps:
    ///
    /// 1. Creates a test directory for a Git-like repository if it doesn't exist.
    /// 2. Creates a sample commit and sets the HEAD file to point to a branch ("main").
    /// 3. Calls the `checkout_commit_detached` function with an existing commit ID, which should force
    ///    the change to a detached commit by updating the HEAD reference.
    /// 4. Verifies that the HEAD reference is correctly updated to represent a detached commit.
    ///
    /// This test ensures that the `checkout_commit_detached` function correctly handles changing to a
    /// specific commit in detached mode.
    ///
    #[test]
    fn test_checkout_commit_detached_references() {
        // Create a test directory if it doesn't exist
        if !Path::new(T).exists() {
            fs::create_dir_all(T).expect("Failed to create test directory");
        }

        // Create a sample commit and set the HEAD file
        let objects_dir = Path::new(T).join("objects");
        let commit_id = "commit_id";
        let commit_file = objects_dir.join(&commit_id);
        fs::create_dir_all(&commit_file.parent().unwrap()).expect("Failed to create dirs");
        fs::write(&commit_file, "commit_content").expect("Failed to write commit object");

        let head_file = Path::new(T).join("HEAD");
        fs::write(&head_file, "ref: refs/heads/main\n").expect("Failed to write HEAD file");

        //Write a refs/heads/main file
        let refs_dir = Path::new(T).join("refs").join("heads");
        fs::create_dir_all(&refs_dir).expect("Failed to create dirs");
        let branch_ref_file = refs_dir.join("main");
        fs::File::create(&branch_ref_file).expect("Failed to create file");
        let mut old_branch_file = fs::File::create(branch_ref_file).unwrap();
        write_reference_value(&mut old_branch_file, "commit_id").unwrap();

        // Execute the checkout_commit_detached function with a commit in detached mode
        let result = checkout_commit_detached_references(T, commit_id);

        assert!(result.is_ok());
        // Verify that the HEAD file has been updated to point to the commit in detached mode
        let head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(head_contents, format!("{} (commit)\n", commit_id));
    }

    /// Unit test for the `create_or_reset_branch` function.
    ///
    /// This test validates the behavior of the `create_or_reset_branch` function when creating or
    /// resetting a branch in a Git-like repository. It follows these steps:
    ///
    /// 1. Creates a test directory for a Git-like repository if it doesn't exist.
    /// 2. Creates a sample branch and sets the HEAD file to point to another branch ("other_branch").
    /// 3. Calls the `create_or_reset_branch` function with an existing branch name, which should reset
    ///    the branch by updating the HEAD reference.
    /// 4. Verifies that the HEAD reference is correctly updated to point to the specified branch.
    /// 5. Calls the `create_or_reset_branch` function with a new branch name, which should create a new
    ///    branch and update the HEAD reference.
    /// 6. Verifies that the new branch has been created and that the HEAD reference is correctly updated.
    ///
    /// This test ensures that the `create_or_reset_branch` function correctly handles branch creation
    /// and resetting by updating the HEAD reference.
    ///
    #[test]
    #[ignore]
    fn test_create_or_reset_branch() {
        // Create a test directory if it does not exist
        if !Path::new(TEST_GIT).exists() {
            fs::create_dir_all(TEST_GIT).expect("Failed to create test directory");
        }
        // Create a sample branch and set the HEAD file to point to another branch
        let refs_dir = Path::new(TEST_GIT).join("refs").join("heads");
        let branch_name = "other_branch";
        let branch_ref_file = refs_dir.join(branch_name);
        fs::create_dir_all(&branch_ref_file.parent().unwrap()).expect("Failed to create dirs");
        fs::write(&branch_ref_file, "commit_id").expect("Failed to write branch reference");

        let head_file = Path::new(TEST_GIT).join("HEAD");
        fs::write(&head_file, format!("ref: refs/heads/other_branch\n"))
            .expect("Failed to write HEAD file");

        // Execute the create_or_reset_branch function with an existing branch
        let _ = create_or_reset_branch(Path::new(TEST_GIT), TEST_GIT, branch_name);

        // Verify that the HEAD file has been updated to point to the new branch
        let head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(head_contents, format!("ref: refs/heads/{}\n", branch_name));

        // Execute the create_or_reset_branch function with a non-existent branch
        let _ = create_or_reset_branch(Path::new(TEST_GIT), TEST_GIT, "new_branch");

        // Verify that the new branch has been created, and that the HEAD file has been updated
        assert!(Path::new(TEST_GIT)
            .join("refs")
            .join("heads")
            .join("new_branch")
            .exists());
        let new_head_contents = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(new_head_contents, format!("ref: refs/heads/new_branch\n"));
    }

    fn prepare_dir(git_dir: &str) {
        //Create a refs heads directory and write the old branch name
        let refs_dir = Path::new(git_dir).join("refs").join("heads");
        fs::create_dir_all(&refs_dir).expect("Failed to create dirs");

        //Create an objects directory and write a commit
        let objects_dir = Path::new(git_dir).join("objects");
        fs::create_dir_all(&objects_dir).expect("Failed to create dirs");

        //Create a HEAD file and write the old branch name
        let head_file = Path::new(git_dir).join("HEAD");
        fs::File::create(&head_file).expect("Failed to create file");
        let head_contents = format!("ref: refs/heads/master");
        fs::write(&head_file, head_contents).expect("Failed to write HEAD file");

        //Create an index file
        let index_file = Path::new(git_dir).join("index");
        fs::File::create(&index_file).expect("Failed to create file");
    }

    fn prepare_commit_1(test_dir: &str) {
        let file1 = Path::new(test_dir).join("archivo.txt");
        let file2 = Path::new(test_dir).join("otro_archivo.txt");

        fs::File::create(&file1).expect("Failed to create file");
        fs::File::create(&file2).expect("Failed to create file");

        fs::write(&file1, "Hola").expect("Failed to write file");
        fs::write(&file2, "Chau").expect("Failed to write file");

        let git_dir = test_dir.to_string() + "/.mgit";
        let index_path = git_dir.to_string() + "/index";

        let _ = add::add(
            &(test_dir.to_string() + "/archivo.txt"),
            &index_path,
            &git_dir,
            "",
            None,
        );

        let _ = add::add(
            &(test_dir.to_string() + "/otro_archivo.txt"),
            &index_path,
            &git_dir,
            "",
            None,
        );
    }

    fn prepare_commit_2(test_dir: &str) {
        let file1 = Path::new(test_dir).join("archivo.txt");

        let dir = Path::new(test_dir).join("otro_dir");
        fs::create_dir_all(&dir).expect("Failed to create dirs");
        let file3 = Path::new(test_dir).join("otro_dir/nuevo_archivo.txt");

        fs::File::create(&file1).expect("Failed to create file");
        fs::File::create(&file3).expect("Failed to create file");

        fs::write(&file1, "Este es otro hola").expect("Failed to write file");
        fs::write(&file3, "Adios").expect("Failed to write file");

        let git_dir = test_dir.to_string() + "/.mgit";
        let index_path = git_dir.to_string() + "/index";

        let _ = add::add(
            &(test_dir.to_string() + "/archivo.txt"),
            &index_path,
            &git_dir,
            "",
            None,
        );

        let _ = add::add(
            &(test_dir.to_string() + "/otro_dir/nuevo_archivo.txt"),
            &index_path,
            &git_dir,
            "",
            None,
        );
    }

    #[test]
    fn test_replace_working_tree() {
        // Create a test directory if it doesn't exist
        if !Path::new("tests/checkout").exists() {
            fs::create_dir_all("tests/checkout").expect("Failed to create test directory");
        }
        prepare_dir("tests/checkout/.mgit");
        prepare_commit_1("tests/checkout");
        let old_commmit = commit::new_commit("tests/checkout/.mgit", "Hola", "").unwrap();
        prepare_commit_2("tests/checkout");
        let new_commit = commit::new_commit("tests/checkout/.mgit", "Hola", "").unwrap();

        replace_working_tree("tests/checkout/.mgit", "", &new_commit, &old_commmit).unwrap();

        let file1 = Path::new("tests/checkout/archivo.txt");
        let file1_content = fs::read_to_string(&file1).expect("Failed to read file");
        assert_eq!(file1_content, "Hola");

        let file3 = Path::new("tests/checkout/otro_dir/nuevo_archivo.txt");
        assert!(!file3.exists());

        assert!(!Path::new("tests/checkout/otro_dir").exists());

        fs::remove_dir_all("tests/checkout").expect("Failed to delete directory");
    }

    #[test]
    fn test_create_and_checkout_branch() {
        // Create a test directory if it doesn't exist
        if !Path::new("tests/checkout2").exists() {
            fs::create_dir_all("tests/checkout2").expect("Failed to create test directory");
        }
        prepare_dir("tests/checkout2/.mgit");
        prepare_commit_1("tests/checkout2");
        let old_commmit = commit::new_commit("tests/checkout2/.mgit", "Hola", "").unwrap();

        let git_dir_path = Path::new("tests/checkout2/.mgit");
        create_and_checkout_branch(git_dir_path, "", "new_branch").unwrap();

        let branch_ref_file = git_dir_path.join("refs").join("heads").join("new_branch");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, old_commmit);

        fs::remove_dir_all("tests/checkout2").expect("Failed to delete directory");
    }

    #[test]
    fn test_create_branch_and_return_to_old_one() {
        // Create a test directory if it doesn't exist
        if !Path::new("tests/checkout3").exists() {
            fs::create_dir_all("tests/checkout3").expect("Failed to create test directory");
        }
        prepare_dir("tests/checkout3/.mgit");
        prepare_commit_1("tests/checkout3");
        let old_commmit = commit::new_commit("tests/checkout3/.mgit", "Hola", "").unwrap();

        let git_dir_path = Path::new("tests/checkout3/.mgit");
        create_and_checkout_branch(git_dir_path, "", "new_branch").unwrap();

        let branch_ref_file = git_dir_path.join("refs").join("heads").join("new_branch");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, old_commmit);

        create_and_checkout_branch(git_dir_path, "", "master").unwrap();

        let branch_ref_file = git_dir_path.join("refs").join("heads").join("master");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, old_commmit);

        fs::remove_dir_all("tests/checkout3").expect("Failed to delete directory");
    }

    #[test]
    fn test_checkout_commit_and_return_to_old_branch() {
        // Create a test directory if it doesn't exist
        if !Path::new("tests/checkout4").exists() {
            fs::create_dir_all("tests/checkout4").expect("Failed to create test directory");
        }
        prepare_dir("tests/checkout4/.mgit");
        prepare_commit_1("tests/checkout4");
        let old_commmit = commit::new_commit("tests/checkout4/.mgit", "Hola", "").unwrap();

        let git_dir_path = Path::new("tests/checkout4/.mgit");
        create_and_checkout_branch(git_dir_path, "", "new_branch").unwrap();

        let branch_ref_file = git_dir_path.join("refs").join("heads").join("new_branch");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, old_commmit);

        prepare_commit_2("tests/checkout4");
        let new_commit = commit::new_commit("tests/checkout4/.mgit", "Hola", "").unwrap();
        create_and_checkout_branch(git_dir_path, "", "master").unwrap();

        let branch_ref_file = git_dir_path.join("refs").join("heads").join("master");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, old_commmit);

        //Check that new_branch has the correct commit
        let branch_ref_file = git_dir_path.join("refs").join("heads").join("new_branch");
        let mut file = fs::File::open(&branch_ref_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, new_commit);

        fs::remove_dir_all("tests/checkout4").expect("Failed to delete directory");
    }
}
