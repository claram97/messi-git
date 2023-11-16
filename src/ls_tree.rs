use std::{io, hash};
use crate::tree_handler;

pub fn ls_tree_no_args(hash: &str, git_dir: &str) -> io::Result<()> {
    match tree_handler::load_tree_from_commit(hash, git_dir) {
        Ok(tree) => {
            tree.print_tree(&mut io::stdout());
            Ok(())
        },
        Err(_) => {
            match tree_handler::load_tree_from_file(hash, git_dir) {
                Ok(tree) => {
                    tree.print_tree(&mut io::stdout());
                    Ok(())
                },
                Err(_) => {
                    Err(io::Error::new(io::ErrorKind::Other, "Not a tree"))
                }
            }
        }
    }
}

pub fn ls_tree_recursive(hash: &str, git_dir: &str) -> io::Result<()> {
    match tree_handler::load_tree_from_commit(hash, git_dir) {
        Ok(tree) => {
            tree.print_tree_recursive_no_trees(&mut io::stdout());
            Ok(())
        },
        Err(_) => {
            match tree_handler::load_tree_from_file(hash, git_dir) {
                Ok(tree) => {
                    tree.print_tree_recursive_no_trees(&mut io::stdout());
                    Ok(())
                },
                Err(_) => {
                    Err(io::Error::new(io::ErrorKind::Other, "Not a tree"))
                }
            }
        }
    }

}