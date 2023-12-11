use std::{fs, io};

use crate::commit::is_merge_commit;
use crate::configuration::{GIT_DIR, LOGGER_COMMANDS_FILE};
use crate::logger::Logger;
use crate::tree_handler::Tree;
use crate::utils::get_current_time;
use crate::{
    branch, commit, tree_handler,
    utils::{self, get_git_ignore_path},
};
use std::io::Write;

/// Checks if a branch fast-forwards to a common commit.
///
/// Determines if the branch's commit is the same as the common commit, indicating a
/// fast-forward scenario.
///
/// # Arguments
///
/// * `our_branch_commit` - The commit hash of the branch.
/// * `common_commit` - The commit hash of the common ancestor.
///
/// # Returns
///
/// Returns `true` if the branch fast-forwards to the common commit, otherwise `false`.
///
pub fn is_fast_forward(our_branch_commit: &str, common_commit: &str) -> bool {
    if our_branch_commit == common_commit {
        return true;
    }
    false
}

//Given two commits, finds the first common ancestor between them.
//Returns an error if no common ancestor is found. (Thing that should never happen)
pub fn find_common_ancestor(
    our_branch_commit: &str,
    their_branch_commit: &str,
    git_dir: &str,
) -> io::Result<String> {
    let our_commit_parents = utils::get_branch_commit_history(our_branch_commit, git_dir)?;
    let their_commit_parents = utils::get_branch_commit_history_set(their_branch_commit, git_dir)?;

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
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let old_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;
    let new_tree = tree_handler::load_tree_from_commit(&their_commit, git_dir)?;
    old_tree.delete_directories(root_dir)?;
    new_tree.create_directories(root_dir, git_dir)?;
    let index_path = utils::get_index_file_path(git_dir);
    let new_index_file_contents =
        new_tree.build_index_file_from_tree(&index_path, git_dir, &get_git_ignore_path(git_dir))?;
    new_index_file_contents.write_file()?;
    branch::update_branch_commit_hash(our_branch, &their_commit, git_dir)?;
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
) -> io::Result<Vec<String>> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;
    let our_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;
    let their_tree = tree_handler::load_tree_from_commit(&their_commit, git_dir)?;
    let (new_tree, conflicting_paths) = tree_handler::merge_trees(&our_tree, &their_tree, git_dir)?;
    our_tree.delete_directories(root_dir)?;
    new_tree.create_directories(root_dir, git_dir)?;
    let index_path = utils::get_index_file_path(git_dir);
    let new_index_file_contents =
        new_tree.build_index_file_from_tree(&index_path, git_dir, &get_git_ignore_path(git_dir))?;
    new_index_file_contents.write_file()?;
    Ok(conflicting_paths)
}

/// Logs the 'git merge' command with the specified branch names, Git directory, and root directory.
///
/// This function logs the 'git merge' command with the provided our branch, their branch, Git directory,
/// and root directory to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `our_branch` - A string representing the name of our branch.
/// * `their_branch` - A string representing the name of their branch.
/// * `git_dir` - A string representing the path to the Git directory.
/// * `root_dir` - A string representing the path to the root directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_merge(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git merge': Our Branch '{}', Their Branch '{}', Git Directory '{}', Root Directory '{}', {}",
        our_branch,
        their_branch,
        git_dir,
        root_dir,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
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
) -> io::Result<(String, Vec<String>)> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_ancestor = find_common_ancestor(&our_commit, &their_commit, git_dir)?;
    if is_fast_forward(&our_commit, &common_ancestor) {
        fast_forward_merge(our_branch, their_branch, git_dir, root_dir)?;
        let tuple = (our_commit, vec![]);
        log_merge(our_branch, their_branch, git_dir, root_dir)?;
        Ok(tuple)
    } else {
        let conflicting_paths = two_way_merge(our_branch, their_branch, git_dir, root_dir)?;
        let commit_message = format!("Merge branch '{}'", their_branch);
        let hash =
            commit::new_merge_commit(git_dir, &commit_message, &our_commit, &their_commit, "")?;
        let tuple = (hash, conflicting_paths);
        log_merge(our_branch, their_branch, git_dir, root_dir)?;

        Ok(tuple)
    }
}

/// Given two branches, merges `our_branch` with `their_branch`.
/// This function is used for the UI, where the user can choose to merge or not.
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
pub fn git_merge_for_ui(
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
    root_dir: &str,
) -> io::Result<Vec<String>> {
    let our_commit = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_commit = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_ancestor = find_common_ancestor(&our_commit, &their_commit, git_dir)?;
    if is_fast_forward(&our_commit, &common_ancestor) {
        fast_forward_merge(our_branch, their_branch, git_dir, root_dir)?;
        log_merge(our_branch, their_branch, git_dir, root_dir)?;
        Ok(vec![])
    } else {
        let conflicting_paths = two_way_merge(our_branch, their_branch, git_dir, root_dir)?;
        // Create a MERGE_HEAD file
        let mut merge_head_file = fs::File::create(format!("{}/{}/MERGE_HEAD", root_dir, GIT_DIR))?;
        merge_head_file.write_all(their_commit.as_bytes())?;

        // Create a merge_index file where all the conflicts are written
        let mut merge_index_file =
            fs::File::create(format!("{}/{}/MERGE_INDEX", root_dir, GIT_DIR))?;
        for path in conflicting_paths.iter() {
            merge_index_file.write_all(path.as_bytes())?;
            merge_index_file.write_all(b"\n")?;
        }
        log_merge(our_branch, their_branch, git_dir, root_dir)?;
        Ok(conflicting_paths)
    }
}

/// Merge a remote branch into the current local branch in a Git repository.
///
/// This function performs a merge operation by combining the changes from a remote branch into the
/// current local branch of a Git repository. The merge process involves updating the index and working
/// tree to reflect the new merged state.
///
/// # Arguments
///
/// * `branch`: A string representing the name of the local branch to be merged.
/// * `remote_hash`: A string representing the commit hash of the remote branch to be merged.
/// * `git_dir`: A string representing the path to the Git repository directory.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the merge operation was successful. If successful,
/// `Ok(())` is returned; otherwise, an error is returned.
///
pub fn merge_remote_branch(branch: &str, remote_hash: &str, git_dir: &str) -> io::Result<Tree> {
    let our_commit = branch::get_branch_commit_hash(branch, git_dir)?;
    let common_ancestor = find_common_ancestor(&our_commit, remote_hash, git_dir)?;
    if is_fast_forward(&our_commit, &common_ancestor) {
        let remote_tree = tree_handler::load_tree_from_commit(remote_hash, git_dir)?;
        let index_path = utils::get_index_file_path(git_dir);
        let new_index_file_contents = remote_tree.build_index_file_from_tree(
            &index_path,
            git_dir,
            &get_git_ignore_path(git_dir),
        )?;
        new_index_file_contents.write_file()?;
        branch::update_branch_commit_hash(branch, remote_hash, git_dir)?;
        Ok(remote_tree)
    } else {
        let our_tree = tree_handler::load_tree_from_commit(&our_commit, git_dir)?;
        let remote_tree = tree_handler::load_tree_from_commit(remote_hash, git_dir)?;
        let (new_tree, _conflicts) = tree_handler::merge_trees(&our_tree, &remote_tree, git_dir)?;
        let index_path = utils::get_index_file_path(git_dir);
        let new_index_file_contents = new_tree.build_index_file_from_tree(
            &index_path,
            git_dir,
            &get_git_ignore_path(git_dir),
        )?;
        new_index_file_contents.write_file()?;
        tree_handler::build_tree_from_index(&index_path, git_dir, &get_git_ignore_path(git_dir))
    }
}

/// Merges a pull request by combining changes from the pull request commit
/// with the base commit. Handles conflicts if any and creates a new commit.
///
/// # Arguments
///
/// * `base_commit` - The commit hash of the base branch.
/// * `pull_request_commit` - The commit hash of the pull request branch.
/// * `pull_request_branch` - The name of the pull request branch.
/// * `git_dir` - The path to the Git directory.
/// * `target_branch` - The name of the target branch for the merge.
///
/// # Returns
///
/// Returns a `Result` with either the new commit hash on success or an
/// `io::Error` containing information about conflicts if any.
///
fn merge_pull_request(
    base_commit: &str,
    pull_request_commit: &str,
    pull_request_branch: &str,
    git_dir: &str,
    target_branch: &str,
) -> Result<String, io::Error> {
    let base_tree = tree_handler::load_tree_from_commit(base_commit, git_dir)?;
    let pull_request_tree = tree_handler::load_tree_from_commit(pull_request_commit, git_dir)?;
    let (new_tree, conflicts) = tree_handler::merge_trees(&base_tree, &pull_request_tree, git_dir)?;
    if conflicts.is_empty() {
        let commit_message = format!("Merge pull request #{}", pull_request_branch);
        let commit_hash = commit::new_pr_merge_commit(
            git_dir,
            &commit_message,
            base_commit,
            pull_request_commit,
            &new_tree,
            target_branch,
        )?;
        Ok(commit_hash)
    } else {
        let mut conflicting_paths = String::new();
        for path in conflicts.iter() {
            conflicting_paths.push_str(path);
            conflicting_paths.push('\n');
        }
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            format!(
                "Conflicts found when trying to merge the Pull Request:\n{}",
                conflicting_paths
            ),
        ))
    }
}

/// Checks if a pull request can be fast-forwarded into the base branch.
///
/// # Arguments
///
/// * `base_commit` - The commit hash of the base branch.
/// * `pull_request_commit` - The commit hash of the pull request branch.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` with a boolean indicating whether the pull request
/// can be fast-forwarded into the base branch or not. Returns `Ok(true)` if
/// fast-forward is possible, and `Ok(false)` otherwise. Returns an `io::Error`
/// if there is an issue with the Git operations.
///
pub fn is_pr_fast_forward(
    base_commit: &str,
    pull_request_commit: &str,
    git_dir: &str,
) -> io::Result<bool> {
    let common_ancestor = find_common_ancestor(base_commit, pull_request_commit, git_dir)?;
    if is_fast_forward(base_commit, &common_ancestor) {
        Ok(true)
    } else if is_merge_commit(pull_request_commit, git_dir)? {
        let parents = commit::get_merge_parents(pull_request_commit, git_dir)?;
        if parents.contains(&base_commit.to_string()) {
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

// A function that merges a pull request into the base branch. It does not update the working directory.
// It returns the hash of the new commit. The base will take the role of our_branch and the pull request branch will be their_branch.
// If there are no conflicts, we will create a new commit with the changes of both branches.
pub fn git_merge_for_pull_request(
    base_branch: &str,
    pull_request_branch: &str,
    git_dir: &str,
) -> io::Result<String> {
    let base_commit = branch::get_branch_commit_hash(base_branch, git_dir)?;
    let pull_request_commit = branch::get_branch_commit_hash(pull_request_branch, git_dir)?;

    if is_pr_fast_forward(&base_commit, &pull_request_commit, git_dir)? {
        let pull_request_tree = tree_handler::load_tree_from_commit(&pull_request_commit, git_dir)?;
        let commit_message = format!("Merge pull request #{}", pull_request_branch);
        let commit_hash = commit::new_pr_merge_commit(
            git_dir,
            &commit_message,
            &base_commit,
            &pull_request_commit,
            &pull_request_tree,
            base_branch,
        )?;
        Ok(commit_hash)
    } else {
        merge_pull_request(
            &base_commit,
            &pull_request_commit,
            pull_request_branch,
            git_dir,
            base_branch,
        )
    }
}

#[cfg(test)]
mod tests {

    use std::{
        fs,
        io::{Read, Write},
    };

    use crate::{add, commit};

    use super::*;
    const NAME_OF_GIT_DIRECTORY_1: &str = "tests/merge/test_common_ancestor_1/.mgit";
    const NAME_OF_GIT_DIRECTORY_2: &str = "tests/merge/test_common_ancestor_2/.mgit";
    const NAME_OF_GIT_DIRECTORY_3: &str = "tests/merge/test_fast_forward_merge/.mgit";
    const NAME_OF_GIT_DIRECTORY_4: &str = "tests/merge/test_true_merge/.mgit";
    const NAME_OF_GIT_DIRECTORY_5: &str = "tests/merge/test_conflict_merge/.mgit";

    #[test]
    fn is_fast_forward_returns_true_when_our_branch_commit_is_equal_to_common_commit() {
        let our_branch_commit = "their_branch_commit";
        let common_commit = "their_branch_commit";

        assert_eq!(is_fast_forward(our_branch_commit, common_commit), true);
    }

    #[test]
    fn is_fast_forward_returns_false_when_our_branch_commit_is_not_equal_to_common_commit() {
        let our_branch_commit = "their_branch_commit";
        let common_commit = "common_commit";

        assert_eq!(is_fast_forward(our_branch_commit, common_commit), false);
    }

    fn create_mock_git_dir(git_dir: &str) {
        fs::create_dir_all(&git_dir).unwrap();
        let objects_dir = format!("{}/objects", git_dir);
        fs::create_dir_all(&objects_dir).unwrap();
        let refs_dir = format!("{}/refs/heads", git_dir);
        fs::create_dir_all(&refs_dir).unwrap();
        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file.write_all(b"ref: refs/heads/main").unwrap();
        let main_file_path = format!("{}/main", refs_dir);
        let mut main_file = fs::File::create(&main_file_path).unwrap();
        main_file.write_all(b"hash_del_commit_anterior").unwrap();
    }

    #[test]
    fn find_common_ancestor_returns_the_first_common_ancestor_between_two_commits_ff() {
        let git_dir = NAME_OF_GIT_DIRECTORY_1;
        create_mock_git_dir(NAME_OF_GIT_DIRECTORY_1);

        let index_file_path = format!("{}/index", git_dir);
        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c")
            .unwrap();

        let commit_message = "Initial commit";
        let commit_1_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let branch_name = "test";
        let _ = branch::create_new_branch(&git_dir, "test", None, &mut io::stdout());

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", branch_name).as_bytes())
            .unwrap();

        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/hello.c")
            .unwrap();

        let commit_message = "Second commit";
        let _commit_2_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/hello.c\nf454180c3a3b6182000a3ef25b3ef2e10cb10234 src/bye.c")
            .unwrap();

        let commit_message = "Third commit";
        let commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let common_ancestor =
            find_common_ancestor(&commit_1_hash, &commit_3_hash, &git_dir).unwrap();
        assert_eq!(common_ancestor, commit_1_hash);
        fs::remove_dir_all(NAME_OF_GIT_DIRECTORY_1).unwrap();
    }

    #[test]
    fn find_common_ancestor_returns_the_first_common_ancestor_between_two_commits_true_merge() {
        let git_dir = NAME_OF_GIT_DIRECTORY_2;
        create_mock_git_dir(NAME_OF_GIT_DIRECTORY_2);

        let index_file_path = format!("{}/index", git_dir);
        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c")
            .unwrap();

        let commit_message = "Initial commit";
        let _commit_1_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/hello.c")
            .unwrap();

        let commit_message = "Second commit";
        let commit_2_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let branch_name = "branch1";
        let _ = branch::create_new_branch(&git_dir, branch_name, None, &mut io::stdout());

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{branch_name}").as_bytes())
            .unwrap();

        let mut index_file = fs::File::create(&index_file_path).unwrap();
        index_file
            .write_all(b"3ed3021d73efc1e9c5f31cf87934e49cd201a72c src/main.c\ne4482842d2f8e960ccb99c3026f1210ea2b1d24e src/hello.c\nf454180c3a3b6182000a3ef25b3ef2e10cb10234 src/bye.c")
            .unwrap();

        let commit_message = "Third commit";
        let commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/main").as_bytes())
            .unwrap();

        let commit_message = "Fourth commit";
        let commit_4_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let common_ancestor =
            find_common_ancestor(&commit_3_hash, &commit_4_hash, &git_dir).unwrap();
        assert_eq!(common_ancestor, commit_2_hash);

        fs::remove_dir_all(NAME_OF_GIT_DIRECTORY_2).unwrap();
    }

    #[test]
    fn test_fast_forward_merge_works_correctly() {
        let git_dir = NAME_OF_GIT_DIRECTORY_3;
        create_mock_git_dir(NAME_OF_GIT_DIRECTORY_3);

        let root_dir = "tests/merge/test_fast_forward_merge";
        fs::create_dir_all(format!("{}/src", root_dir)).unwrap();

        let file_1_path = format!("{}/src/main.c", root_dir);
        let mut file = fs::File::create(&file_1_path).unwrap();
        file.write_all(b"int main() { return 0; }").unwrap();
        let file_2_path = format!("{}/src/hello.c", root_dir);
        let mut file = fs::File::create(&file_2_path).unwrap();
        file.write_all(b"int hello() { return 0; }").unwrap();
        let index_file_path = format!("{}/.mgit/index", root_dir);
        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(
            "tests/merge/test_fast_forward_merge/src/hello.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_3,
            "",
            None,
        )
        .unwrap();
        add::add(
            "tests/merge/test_fast_forward_merge/src/main.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_3,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Initial commit";
        let commit_1_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let branch_name = "branch";
        let _ = branch::create_new_branch(&git_dir, branch_name, None, &mut io::stdout());

        let head_file_path = format!("{}/.mgit/HEAD", root_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", branch_name).as_bytes())
            .unwrap();

        let file_3_path = format!("{}/src/bye.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { return 0; }").unwrap();
        add::add(
            "tests/merge/test_fast_forward_merge/src/bye.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_3,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_1_path = format!("{}/src/main.c", root_dir);
        let mut file = fs::File::create(&file_1_path).unwrap();
        file.write_all(b"int main() { printf('hola'); return 1; }")
            .unwrap();
        add::add(
            "tests/merge/test_fast_forward_merge/src/main.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_3,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Third commit";
        let commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_eq!(main_branch_hash, commit_1_hash);

        git_merge("main", "branch", &git_dir, "").unwrap();
        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_eq!(main_branch_hash, commit_3_hash);

        let file_1_path = format!("{}/src/main.c", root_dir);
        let mut file = fs::File::open(&file_1_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int main() { printf('hola'); return 1; }");

        let file_2_path = format!("{}/src/hello.c", root_dir);
        let mut file = fs::File::open(&file_2_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int hello() { return 0; }");

        let file_3_path = format!("{}/src/bye.c", root_dir);
        let mut file = fs::File::open(&file_3_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int bye() { return 0; }");

        fs::remove_dir_all(NAME_OF_GIT_DIRECTORY_3).unwrap();
        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn test_true_merge_works_correctly_no_conflicts() {
        let git_dir = NAME_OF_GIT_DIRECTORY_4;
        create_mock_git_dir(NAME_OF_GIT_DIRECTORY_4);

        let root_dir = "tests/merge/test_true_merge";
        fs::create_dir_all(format!("{}/src", root_dir)).unwrap();

        let file_1_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::create(&file_1_path).unwrap();
        file.write_all(b"int main() { return 0; }").unwrap();
        let file_2_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::create(&file_2_path).unwrap();
        file.write_all(b"int hello() { return 0; }").unwrap();
        let index_file_path = format!("{}/.mgit/index", root_dir);
        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(
            "tests/merge/test_true_merge/src/2.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();
        add::add(
            "tests/merge/test_true_merge/src/1.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Initial commit";
        let commit_1_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let branch_name = "branch";
        let _ = branch::create_new_branch(&git_dir, branch_name, None, &mut io::stdout());

        let head_file_path = format!("{}/.mgit/HEAD", root_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", branch_name).as_bytes())
            .unwrap();

        let index_file_path = format!("{}/.mgit/index", root_dir);
        let _ = fs::File::create(&index_file_path).unwrap();

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { return 0; }").unwrap();
        add::add(
            "tests/merge/test_true_merge/src/3.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_4_path = format!("{}/src/4.c", root_dir);
        let mut file = fs::File::create(&file_4_path).unwrap();
        file.write_all(b"int prueba() { return 0; }").unwrap();
        add::add(
            "tests/merge/test_true_merge/src/4.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Third commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/.mgit/HEAD", root_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/main").as_bytes())
            .unwrap();

        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(
            "tests/merge/test_true_merge/src/2.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();
        add::add(
            "tests/merge/test_true_merge/src/1.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();

        let file_5_path = format!("{}/src/5.c", root_dir);
        let mut file = fs::File::create(&file_5_path).unwrap();
        file.write_all(b"int otro() { return 0; }").unwrap();
        let index_file_path = format!("{}/.mgit/index", root_dir);
        add::add(
            "tests/merge/test_true_merge/src/5.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_4,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Fourth commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_ne!(main_branch_hash, commit_1_hash);

        let branch_hash = branch::get_branch_commit_hash("branch", &git_dir).unwrap();
        assert_ne!(branch_hash, commit_1_hash);

        let file_6_path = format!("{}/src/placeholder.c", root_dir);
        let mut file = fs::File::create(&file_6_path).unwrap();
        file.write_all(b"int placeholder() { return 0; }").unwrap();

        for i in 1..6 {
            let file_path = format!("{}/src/{}.c", root_dir, i);
            fs::remove_file(file_path).unwrap();
        }

        let merge_commit_hash = git_merge("main", "branch", &git_dir, "").unwrap().0;

        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_eq!(main_branch_hash, merge_commit_hash);

        let file_1_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::open(&file_1_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int main() { return 0; }");

        let file_2_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::open(&file_2_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int hello() { return 0; }");

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::open(&file_3_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int bye() { return 0; }");

        let file_4_path = format!("{}/src/4.c", root_dir);
        let mut file = fs::File::open(&file_4_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int prueba() { return 0; }");

        let file_5_path = format!("{}/src/5.c", root_dir);
        let mut file = fs::File::open(&file_5_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int otro() { return 0; }");

        fs::remove_dir_all(NAME_OF_GIT_DIRECTORY_4).unwrap();
        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn test_true_merge_works_correctly_with_conflicts() {
        let git_dir = NAME_OF_GIT_DIRECTORY_5;
        create_mock_git_dir(NAME_OF_GIT_DIRECTORY_5);

        let root_dir = "tests/merge/test_conflict_merge";
        fs::create_dir_all(format!("{}/src", root_dir)).unwrap();

        let file_1_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::create(&file_1_path).unwrap();
        file.write_all(b"int main() { return 0; }").unwrap();
        let file_2_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::create(&file_2_path).unwrap();
        file.write_all(b"int hello() { return 0; }").unwrap();
        let index_file_path = format!("{}/.mgit/index", root_dir);
        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(
            "tests/merge/test_conflict_merge/src/2.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();
        add::add(
            "tests/merge/test_conflict_merge/src/1.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Initial commit";
        let commit_1_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let branch_name = "branch";
        let _ = branch::create_new_branch(&git_dir, branch_name, None, &mut io::stdout());

        let head_file_path = format!("{}/.mgit/HEAD", root_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", branch_name).as_bytes())
            .unwrap();

        let _ = fs::File::create(&index_file_path).unwrap();

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { return 0; }").unwrap();
        add::add(
            "tests/merge/test_conflict_merge/src/3.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_4_path = format!("{}/src/4.c", root_dir);
        let mut file = fs::File::create(&file_4_path).unwrap();
        file.write_all(b"int prueba() {\n return 0;\n}").unwrap();
        let index_file_path = format!("{}/.mgit/index", root_dir);
        add::add(
            "tests/merge/test_conflict_merge/src/4.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Third commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/.mgit/HEAD", root_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/main").as_bytes())
            .unwrap();

        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(
            "tests/merge/test_conflict_merge/src/2.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();
        add::add(
            "tests/merge/test_conflict_merge/src/1.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { print('hola'); return -1; }")
            .unwrap();

        add::add(
            "tests/merge/test_conflict_merge/src/3.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let file_4_path = format!("{}/src/4.c", root_dir);
        let mut file = fs::File::create(&file_4_path).unwrap();
        file.write_all(b"int prueba_refactor() {\n return 1;\n}")
            .unwrap();

        let index_file_path = format!("{}/.mgit/index", root_dir);
        add::add(
            "tests/merge/test_conflict_merge/src/4.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let file_5_path = format!("{}/src/5.c", root_dir);
        let mut file = fs::File::create(&file_5_path).unwrap();
        file.write_all(b"int otro() { return 0; }").unwrap();

        add::add(
            "tests/merge/test_conflict_merge/src/5.c",
            &index_file_path,
            NAME_OF_GIT_DIRECTORY_5,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Fourth commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_ne!(main_branch_hash, commit_1_hash);

        let branch_hash = branch::get_branch_commit_hash("branch", &git_dir).unwrap();
        assert_ne!(branch_hash, commit_1_hash);

        let file_6_path = format!("{}/src/placeholder.c", root_dir);
        let mut file = fs::File::create(&file_6_path).unwrap();
        file.write_all(b"int placeholder() { return 0; }").unwrap();

        for i in 1..6 {
            let file_path = format!("{}/src/{}.c", root_dir, i);
            fs::remove_file(file_path).unwrap();
        }

        let merge_commit_hash = git_merge("main", "branch", &git_dir, "").unwrap().0;

        let main_branch_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_eq!(main_branch_hash, merge_commit_hash);

        let file_1_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::open(&file_1_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let file_2_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::open(&file_2_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "int hello() { return 0; }");

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::open(&file_3_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let expected_contents =
            "<<<<<<< int bye() { return 0; }\n>>>>>>> int bye() { print('hola'); return -1; }\n";
        assert_eq!(contents, expected_contents);

        fs::remove_dir_all(NAME_OF_GIT_DIRECTORY_5).unwrap();
        fs::remove_dir_all(root_dir).unwrap();
    }
}
