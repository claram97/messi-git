use crate::configuration::LOGGER_COMMANDS_FILE;
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
    let log_file_path = LOGGER_COMMANDS_FILE;
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

#[cfg(test)]
mod tests {
    use super::*;
    fn create_git_dir(git_dir_path: &str) {
        let _ = std::fs::remove_dir_all(git_dir_path);
        let _ = std::fs::create_dir_all(git_dir_path);
        let _ = std::fs::create_dir_all(git_dir_path.to_string() + "/objects");
    }

    #[test]
    fn test_ls_tree_no_option() {
        let git_dir_path = "tests/ls-tree/test_ls_tree_no_option";
        create_git_dir(git_dir_path);
        let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
        let index_file_content = "00000855bce90795f20fffff5242cc9235000000 a.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 b.c\n000008afba902111fffffa8ebcc70522a3e00000 d.c\n00000128d8c22fc69fffff0d9620ab896b500000 e.c";
        index_file.write_all(index_file_content.as_bytes()).unwrap();
        let mut output = Vec::new();
        let tree = tree_handler::build_tree_from_index(
            "tests/ls-tree/test_ls_tree_no_option/index",
            git_dir_path,
            "",
        )
        .unwrap();
        tree.print_tree(&mut output).unwrap();
        let output_string = String::from_utf8(output).unwrap();
        let expected_output = "100644 blob 00000855bce90795f20fffff5242cc9235000000\ta.txt\n100644 blob 00000c0a42c61e70f66bfffff38fa653b7200000\tb.c\n100644 blob 000008afba902111fffffa8ebcc70522a3e00000\td.c\n100644 blob 00000128d8c22fc69fffff0d9620ab896b500000\te.c\n";
        assert_eq!(output_string, expected_output);

        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn test_ls_tree_r_option() {
        let git_dir_path = "tests/ls-tree/test_ls_tree_r_option";
        create_git_dir(git_dir_path);
        let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
        let index_file_content = "00000855bce90795f20fffff5242cc9235000000 a.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 b.c\n000008afba902111fffffa8ebcc70522a3e00000 d.c\n00000128d8c22fc69fffff0d9620ab896b500000 e.c\n00000855bce90795f20fffff5242cc9235000000 subdir/a.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 subdir/b.c\n000008afba902111fffffa8ebcc70522a3e00000 subdir/d.c\n00000128d8c22fc69fffff0d9620ab896b500000 subdir/e.c";
        index_file.write_all(index_file_content.as_bytes()).unwrap();
        let mut output = Vec::new();
        let tree = tree_handler::build_tree_from_index(
            "tests/ls-tree/test_ls_tree_r_option/index",
            git_dir_path,
            "",
        )
        .unwrap();
        tree.print_tree_recursive_no_trees(&mut output).unwrap();
        let output_string = String::from_utf8(output).unwrap();
        println!("{:#?}", tree);
        // The output should be the same as the previous test, but with the subdirectory files as well
        let expected_output = "100644 blob 00000855bce90795f20fffff5242cc9235000000\ta.txt\n100644 blob 00000c0a42c61e70f66bfffff38fa653b7200000\tb.c\n100644 blob 000008afba902111fffffa8ebcc70522a3e00000\td.c\n100644 blob 00000128d8c22fc69fffff0d9620ab896b500000\te.c\n100644 blob 00000855bce90795f20fffff5242cc9235000000\tsubdir/a.txt\n100644 blob 00000c0a42c61e70f66bfffff38fa653b7200000\tsubdir/b.c\n100644 blob 000008afba902111fffffa8ebcc70522a3e00000\tsubdir/d.c\n100644 blob 00000128d8c22fc69fffff0d9620ab896b500000\tsubdir/e.c\n";
        assert_eq!(output_string, expected_output);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }

    #[test]
    fn test_ls_tree_d_option() {
        let git_dir_path = "tests/ls-tree/test_ls_tree_d_option";
        create_git_dir(git_dir_path);
        let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
        let index_file_content = "00000855bce90795f20fffff5242cc9235000000 a.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 b.c\n000008afba902111fffffa8ebcc70522a3e00000 d.c\n00000128d8c22fc69fffff0d9620ab896b500000 e.c\n00000855bce90795f20fffff5242cc9235000000 subdir/a.txt\n00000c0a42c61e70f66bfffff38fa653b7200000 subdir/b.c\n000008afba902111fffffa8ebcc70522a3e00000 subdir/d.c\n00000128d8c22fc69fffff0d9620ab896b500000 subdir/e.c";
        index_file.write_all(index_file_content.as_bytes()).unwrap();
        let mut output = Vec::new();
        let tree = tree_handler::build_tree_from_index(
            "tests/ls-tree/test_ls_tree_d_option/index",
            git_dir_path,
            "",
        )
        .unwrap();
        tree.print_subtrees(&mut output).unwrap();
        let output_string = String::from_utf8(output).unwrap();
        let expected_output = "040000 tree 3aa73f36ac480cc7ad4393cc02851c4e5a2224e0\tsubdir\n";
        assert_eq!(output_string, expected_output);
        let _ = std::fs::remove_dir_all(git_dir_path);
    }
}
