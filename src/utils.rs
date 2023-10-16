pub(crate) const NAME_OF_GIT_DIRECTORY: &str = ".git";
pub(crate) const NAME_OF_INDEX_FILE: &str = "index-file";
/// Recursively searches for a directory named "name_of_git_directory" in the file system
/// starting from the location specified by "current_dir."
///
/// # Arguments
///
/// * `current_dir`: A mutable reference to a `PathBuf` representing the initial location from which the search begins.
/// * `name_of_git_directory`: The name of the directory being sought.
///
/// # Returns
///
/// This function returns an `Option<String>` containing the path to the found directory as a string if it is found.
/// If the directory is not found, it returns `None`.
///
/// # Example
///
/// ```
///
/// let mut current_dir = env::current_dir();
/// let git_directory_name = ".git";
///
/// if let Some(git_dir_path) = find_git_directory(&mut current_dir, git_directory_name) {
///     println!("Found the Git directory at: {}", git_dir_path);
/// } else {
///     println!("Git directory not found in the search path.");
/// }
/// ```
pub fn find_git_directory(
    current_dir: &mut PathBuf,
    name_of_git_directory: &str,
) -> Option<String> {
    loop {
        let git_dir = current_dir.join(name_of_git_directory);
        if git_dir.exists() && git_dir.is_dir() {
            return Some(git_dir.display().to_string());
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    
}
