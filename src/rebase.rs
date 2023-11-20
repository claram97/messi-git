use std::{io, collections::HashMap};

use crate::{branch, commit, utils, merge, tree_handler::{self, Tree}};


/// Grabs the hashes from the common ancestor, our branch and their branch. Creates a new commit applied to the common ancestor, and then rebase the commits from our branch on top of the new commit.
pub fn create_rebasing_commit(our_commit: &str, rebased_commit: &str, common_ancestor: &str, git_dir: &str, parent_hash: &str) -> io::Result<String> {
    let our_tree = tree_handler::load_tree_from_commit(our_commit, git_dir)?;
    let ancestor_tree = tree_handler::load_tree_from_commit(common_ancestor, git_dir)?;
    let mut rebased_tree = tree_handler::load_tree_from_commit(rebased_commit, git_dir)?;
    
    // Get the paths of the files that haven't been modified between the common ancestor and the rebased commit.
    let files_without_changes_in_rebased: HashMap<String, String> = tree_handler::get_files_without_changes(&ancestor_tree, &rebased_tree).into_iter().collect();
    let files_changed_this_commit = tree_handler::get_files_with_changes(&ancestor_tree, &our_tree);

    // For each file changed this commit, we should check if it wasn't changed between the ancestor and rebase.
    // If so, we should simply update the hash.
    for (hash, path) in files_changed_this_commit {
        if files_without_changes_in_rebased.contains_key(&path) {
            rebased_tree.update_tree(&path, &hash);
        } else {
            // Case where a diff should be done!
        }
    }

    let message = format!("Rebasing commit");
    let new_commit_hash = commit::new_rebase_commit(git_dir, &message, parent_hash, &rebased_tree)?;
    
    Ok(new_commit_hash)
}

pub fn rebase(our_branch: &str, their_branch: &str, git_dir: &str) -> io::Result<()> {
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_commit_ancestor = merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, &git_dir)?;
    let mut our_branch_commits = utils::get_branch_commit_history_until(&our_branch_hash, &git_dir, &common_commit_ancestor)?;
    our_branch_commits.reverse();
    
    let mut our_new_branch_hash = their_branch_hash.clone();
    
    while let Some(commit_hash) = our_branch_commits.pop() {
        our_new_branch_hash = create_rebasing_commit(&commit_hash, &their_branch_hash, &common_commit_ancestor, &git_dir, &our_new_branch_hash)?;
    }

    Ok(())
}