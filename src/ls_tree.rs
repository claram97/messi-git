use std::io;
use crate::tree_handler;
use std::io::Write;

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

    Ok(())
}

