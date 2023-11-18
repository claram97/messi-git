use std::io;
use crate::tree_handler;

/// Performs a ls-tree with no aditional options.
/// The given hash must be either refer to a commit or to a tree.
/// It will list all the blobs and subtrees listed in a tree object
/// 
/// # Arguments
/// 
/// * `hash` - The hash that refers to a tree-like object (either a commit or a tree)
/// * `git_dir` - The path to the git dir
/// 
/// # Errors
/// 
/// This function will fail if:
///     * The hash does not point to a tree-like object
///     * There is an error during a file operation
pub fn ls_tree_no_args(hash: &str, git_dir: &str) -> io::Result<()> {
    match tree_handler::load_tree_from_commit(hash, git_dir) {
        Ok(tree) => {
            tree.print_tree(&mut io::stdout())?;
            Ok(())
        },
        Err(_) => {
            match tree_handler::load_tree_from_file(hash, git_dir) {
                Ok(tree) => {
                    tree.print_tree(&mut io::stdout())?;
                    Ok(())
                },
                Err(_) => {
                    Err(io::Error::new(io::ErrorKind::Other, "Not a tree"))
                }
            }
        }
    }
}

/// Performs a ls-tree with the -r option.
/// The given hash must be either refer to a commit or to a tree.
/// It will list all the blobs in the referred tree and also list the blobs contained in the subtrees of it.
/// 
/// # Arguments
/// 
/// * `hash` - The hash that refers to a tree-like object (either a commit or a tree)
/// * `git_dir` - The path to the git dir
/// 
/// # Errors
/// 
/// This function will fail if:
///     * The hash does not point to a tree-like object
///     * There is an error during a file operation
pub fn ls_tree_recursive(hash: &str, git_dir: &str) -> io::Result<()> {
    match tree_handler::load_tree_from_commit(hash, git_dir) {
        Ok(tree) => {
            tree.print_tree_recursive_no_trees(&mut io::stdout())?;
            Ok(())
        },
        Err(_) => {
            match tree_handler::load_tree_from_file(hash, git_dir) {
                Ok(tree) => {
                    tree.print_tree_recursive_no_trees(&mut io::stdout())?;
                    Ok(())
                },
                Err(_) => {
                    Err(io::Error::new(io::ErrorKind::Other, "Not a tree"))
                }
            }
        }
    }

}