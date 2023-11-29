use crate::logger::Logger;
use crate::tree_handler;
use crate::utils::get_current_time;
use std::io;
use std::io::Write;

/// Logs the 'git ls-tree' command with the specified hash, Git directory, and option.
///
/// This function logs the 'git ls-tree' command with the provided hash, Git directory, and option
/// to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `hash` - A string representing the hash to list in the tree.
/// * `git_dir` - A string representing the path to the Git directory.
/// * `option` - A string representing the option to include in the command.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_ls_tree(hash: &str, git_dir: &str, option: &str) -> io::Result<()> {
    let log_file_path = "logger_commands.txt";
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git ls-tree': Hash '{}', Git Directory '{}', Option '{}', {}",
        hash,
        git_dir,
        option,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// The given hash must be either refer to a commit or to a tree.
/// It will list all the blobs in the referred tree and also list the blobs contained in the subtrees of it.
///
/// # Arguments
///
/// * `hash` - The hash that refers to a tree-like object (either a commit or a tree)
/// * `git_dir` - The path to the git dir
/// * `option` - The ls-tree option (-r, -d, -r-t)
///
/// # Errors
///
/// This function will fail if:
///     * The hash does not point to a tree-like object
///     * There is an error during a file operation
pub fn ls_tree(hash: &str, git_dir: &str, option: &str, output: &mut impl Write) -> io::Result<()> {
    let tree = match tree_handler::load_tree_from_commit(hash, git_dir) {
        Ok(tree) => tree,
        Err(_) => match tree_handler::load_tree_from_file(hash, git_dir) {
            Ok(tree) => tree,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Not a tree")),
        },
    };

    match option {
        "" => tree.print_tree(output)?,
        "-r" => tree.print_tree_recursive_no_trees(output)?,
        "-d" => tree.print_subtrees(output)?,
        "-r-t" => tree.print_tree_recursive(output, git_dir, "")?,
        _ => return Err(io::Error::new(io::ErrorKind::Other, "Invalid option")),
    }
    log_ls_tree(hash, git_dir, option)?;
    Ok(())
}
