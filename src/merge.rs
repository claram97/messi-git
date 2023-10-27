use std::io;

use crate::{branch, tree_handler, utils};

pub fn is_fast_forward(their_branch_commit: &str, common_commit: &str) -> bool {
    if common_commit == their_branch_commit {
        return true;
    }
    false
}

fn fast_forward_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    branch::update_branch_commit_hash(our_branch, their_branch, git_dir)?;
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir).unwrap();
    let old_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;

    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir).unwrap();
    let new_tree = tree_handler::load_tree_from_commit(&their_commit, git_dir)?;

    old_tree.delete_directories(root_dir)?;
    new_tree.create_directories(root_dir, git_dir)?;

    Ok(())
}

pub fn git_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let our_commit_parents = utils::get_commit_parents(&our_commit, git_dir)?;
    let their_commit_parents = utils::get_commit_parents_set(&their_commit, git_dir)?;

    let mut common_ancestor = String::new();
    for parent in our_commit_parents {
        if their_commit_parents.contains(&parent) {
            common_ancestor = parent;
            break;
        }
    }

    if common_ancestor.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No common ancestor found.",
        ));
    }

    if is_fast_forward(&their_commit, &common_ancestor) {
        fast_forward_merge(our_branch, their_branch, git_dir, root_dir)?;
    }
    Ok(())
}
