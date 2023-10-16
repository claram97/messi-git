pub(crate) const NAME_OF_GIT_DIRECTORY: &str = ".git";
pub(crate) const NAME_OF_INDEX_FILE: &str = "index-file";
/**
 * Recursively searches for the Git directory starting from the current directory and moving upwards.
 *
 * # Example
 *
 * ```
 * use std::env;
 *
 * let git_directory = find_git_directory();
 * match git_directory {
 *     Some(dir) => println!("Git directory found at: {}", dir),
 *     None => println!("No Git directory found in the path."),
 * }
 * ```
 *
 * The function returns `Some(path)` if a valid Git directory is found, where `path` is the path to the found Git directory.
 * If no Git directory is found anywhere in the path upwards from the current directory, the function returns `None`.
 */
pub fn find_git_directory() -> Option<String> {
    if let Ok(current_dir) = env::current_dir() {
        let mut current_dir = current_dir;

        loop {
            let git_dir = current_dir.join(NAME_OF_GIT_DIRECTORY);
            if git_dir.exists() && git_dir.is_dir() {
                return Some(git_dir.display().to_string());
            }

            if !current_dir.pop() {
                break;
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    
}