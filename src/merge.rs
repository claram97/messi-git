use std::io;

use crate::{branch, tree_handler, utils::{self, get_git_ignore_path}, commit};

pub fn is_fast_forward(our_branch_commit: &str, common_commit: &str) -> bool {
    if our_branch_commit == common_commit {
        return true;
    }
    false
}

//Given two commits, finds the first common ancestor between them.
//Returns an error if no common ancestor is found. (Thing that should never happen)
fn find_common_ancestor(
    our_branch_commit: &str,
    their_branch_commit: &str,
    git_dir: &str,
) -> io::Result<String> {
    let our_commit_parents = utils::get_branch_commit_history(&our_branch_commit, git_dir)?;
    let their_commit_parents = utils::get_branch_commit_history_set(&their_branch_commit, git_dir)?;

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
    Ok(common_ancestor)
}

/// Given two branches, fast forwards `our_branch` to `their_branch`.
/// This means that `our_branch` will point to the same commit as `their_branch`
/// And the working directory will be updated to match the one of `their_branch`.
fn fast_forward_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    branch::update_branch_commit_hash(our_branch, their_branch, git_dir)?;
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let old_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;

    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;
    let new_tree = tree_handler::load_tree_from_commit(&their_commit, git_dir)?;
    old_tree.delete_directories(root_dir)?;
    new_tree.create_directories(root_dir, git_dir)?;
    let index_path = utils::get_index_file_path(git_dir);
    let new_index_file_contents = new_tree.build_index_file_from_tree(&index_path, git_dir, &get_git_ignore_path(git_dir))?;
    new_index_file_contents.write_file()?;
    Ok(())
}

/// Given two branches, merges `our_branch` with `their_branch`.
/// `our_branch` will point to a new commit that contains the changes of both branches.
/// The working directory will be updated to match the one of the new commit.
/// If there are conflicts, the user will have to resolve them.
fn two_way_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;
    let our_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;
    let their_tree = tree_handler::load_tree_from_commit(&their_commit, git_dir)?;
    let new_tree = tree_handler::merge_trees(&our_tree, &their_tree, git_dir);
    our_tree.delete_directories(root_dir)?;
    new_tree.create_directories(root_dir, git_dir)?;
    let index_path = utils::get_index_file_path(git_dir);
    let new_index_file_contents = new_tree.build_index_file_from_tree(&index_path, git_dir, &get_git_ignore_path(git_dir))?;
    new_index_file_contents.write_file()?;
    Ok(())
}

/// Given two branches, merges `our_branch` with `their_branch`.
/// It will try to do a fast forward merge, if it is not possible, it will do a two way merge.
/// `our_branch` will point to a new commit that contains the changes of both branches.
/// The working directory will be updated to match the changes.
/// If there are conflicts, the user will have to resolve them.
/// 
/// # Arguments
/// * `our_branch` - The name of the branch that will be updated.
/// * `their_branch` - The name of the branch that will be merged with `our_branch`.
/// * `git_dir` - The path to the git directory.
/// * `root_dir` - The path to the root directory.
/// 
/// # Errors
/// Returns an error if the merge fails.
/// 
pub fn git_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_ancestor = find_common_ancestor(&our_commit, &their_commit, git_dir)?;

    if is_fast_forward(&our_commit, &common_ancestor) {
        fast_forward_merge(our_branch, their_branch, git_dir, root_dir)?;
    } else {
        two_way_merge(our_branch, their_branch, git_dir, root_dir)?;
        let commit_message = format!("Merge branch '{}'", their_branch);
        commit::new_commit(git_dir, &commit_message, "")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::{fs, io::Write};

    use crate::commit;

    use super::*;
    const NAME_OF_GIT_DIRECTORY: &str = "tests/merge/test_ffmerge";

    #[test]
    fn is_fast_forward_returns_true_when_our_branch_commit_is_equal_to_common_commit() {
        let our_branch_commit = "their_branch_commit";
        let common_commit = "their_branch_commit";

        assert_eq!(
            is_fast_forward(our_branch_commit, common_commit),
            true
        );
    }

    #[test]
    fn is_fast_forward_returns_false_when_our_branch_commit_is_not_equal_to_common_commit() {
        let our_branch_commit = "their_branch_commit";
        let common_commit = "common_commit";

        assert_eq!(
            is_fast_forward(our_branch_commit, common_commit),
            false
        );
    }
}
