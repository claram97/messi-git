use std::io;

use crate::{branch, commit, utils, merge, tree_handler::{self, Tree}};


/// Grabs the hashes from the common ancestor, our branch and their branch. Creates a new commit applied to the common ancestor, and then rebase the commits from our branch on top of the new commit.
pub fn create_rebasing_commit(our_commit: &str, rebased_commit: &str, common_ancestor: &str, git_dir: &str) -> io::Result<()> {
    let our_tree = tree_handler::load_tree_from_commit(our_commit, git_dir)?;
    let ancestor_tree = tree_handler::load_tree_from_commit(common_ancestor, git_dir)?;
    let rebased_tree = tree_handler::load_tree_from_commit(rebased_commit, git_dir)?;
    
    let diff_files = tree_handler::get_diffs_between_trees(&our_tree, &ancestor_tree, git_dir);
    
    // Now, the diff_files contains the files that have been modified between the common ancestor and our commit.
    // We need to create a new tree with the files from the rebased commit, and the files from the diff_files.
    // Also check if the file had any changes between the common ancestor and the rebased commit.


    Ok(())
}

pub fn rebase(our_branch: &str, their_branch: &str, git_dir: &str) -> io::Result<()> {
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_commit_ancestor = merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, &git_dir)?;
    let mut our_branch_commits = utils::get_branch_commit_history_until(&our_branch_hash, &git_dir, &common_commit_ancestor)?;
    our_branch_commits.reverse();
    
    let our_new_branch_hash = their_branch_hash;
    
    while let Some(commit_hash) = our_branch_commits.pop() {
        
    }



    Ok(())
}