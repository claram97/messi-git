use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{
    cat_file::{self, cat_tree},
    hash_object,
    index::{self},
};

const BLOB_NORMAL_MODE: &str = "100644";
const TREE_MODE: &str = "40000";

//Tree structure
//files is a vector of tuples (file_name, hash)
#[derive(Debug)]
pub struct Tree {
    pub name: String,
    pub files: Vec<(String, String)>,
    pub directories: Vec<Tree>,
}

impl Tree {
    fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    /// Gets a name, if the directory with that name exists, returns a mutable reference to it.
    /// If it does not exist, creates a new directory with that name and returns a mutable reference to it.
    /// The directory is added to the parent's directories vector.
    fn get_or_create_dir(&mut self, name: &str) -> &mut Tree {
        for (i, dir) in self.directories.iter().enumerate() {
            if dir.name == name {
                return &mut self.directories[i];
            }
        }
        let new_dir = Tree::new(name);
        self.directories.push(new_dir);
        let last_dir_index = self.directories.len() - 1;
        &mut self.directories[last_dir_index]
    }

    /// Get a subdir from a tree.
    /// Do not create it if it doesn't exist.
    fn get_subdir(&self, name: &str) -> Option<&Tree> {
        self.directories.iter().find(|&dir| dir.name == name)
    }

    /// Adds the hash and name of a file to the tree
    /// It keeps an alphabetical order.
    fn add_file(&mut self, name: &str, hash: &str) {
        let insert_idx = self
            .files
            .binary_search_by(|(existing_name, _)| existing_name.cmp(&name.to_owned()));

        match insert_idx {
            Ok(idx) | Err(idx) => {
                self.files.insert(idx, (name.to_string(), hash.to_string()));
            }
        }
    }

    /// Returns the depth of the tree
    pub fn get_depth(&self) -> usize {
        let mut max_depth = 0;
        for dir in &self.directories {
            let depth = dir.get_depth();
            if depth > max_depth {
                max_depth = depth;
            }
        }
        max_depth + 1
    }

    fn map_hexa_tuples_to_bytes(hexa_tuples: Vec<(char, char)>) -> Vec<u8> {
        hexa_tuples
            .iter()
            .map(|(a, b)| {
                let a = a.to_digit(16).unwrap() as u8;
                let b = b.to_digit(16).unwrap() as u8;
                (a << 4) | b
            })
            .collect::<Vec<u8>>()
    }

    /// Returns a string that contains all the blobs added to the tree.
    /// The blobs are formatted as "blob {hash} {file_name}\n"
    pub fn tree_blobs_to_string_formatted(&self) -> Vec<(String, String, Vec<u8>)> {
        let mut result = Vec::new();
        for (file_name, hash) in &self.files {
            //Transform the hash from hexa to bytes
            let hash = hash
                .chars()
                .collect::<Vec<char>>()
                .chunks(2)
                .map(|chunk| (chunk[0], chunk[1]))
                .collect::<Vec<(char, char)>>();

            let hash = Self::map_hexa_tuples_to_bytes(hash);
            result.push((BLOB_NORMAL_MODE.to_string(), file_name.to_string(), hash));
        }
        result
    }

    /// Given a path, this function should return the hash correspondent to it in the tree.
    /// The path must be written with the same format as the index file of the directory.
    /// If the path does not exist, it returns None.
    pub fn get_hash_from_path(&self, path: &str) -> Option<String> {
        let mut path = path.split('/').collect::<Vec<&str>>();
        let file_name = match path.pop() {
            Some(file_name) => file_name,
            None => return None,
        };
        let mut current_tree = self;
        while !path.is_empty() {
            current_tree = match current_tree.get_subdir(path.remove(0)) {
                Some(tree) => tree,
                None => return None,
            };
        }
        for (name, hash) in &current_tree.files {
            if name == file_name {
                return Some(hash.to_string());
            }
        }
        None
    }

    /// Given a tree, recreates the directories and files stored in the tree in the working tree.
    pub fn create_directories(&self, parent_dir: &str, git_dir_path: &str) -> io::Result<()> {
        if parent_dir.is_empty() && self.name.is_empty() {
            let dir_path = self.name.to_string();
            for subdirs in &self.directories {
                subdirs.create_directories(&dir_path, git_dir_path)?;
            }
            return Ok(());
        }
        let dir_path = if parent_dir.is_empty() {
            parent_dir.to_string() + &self.name
        } else if self.name.is_empty() {
            parent_dir.to_string()
        } else {
            parent_dir.to_string() + "/" + &self.name
        };

        if !Path::new(&dir_path).exists() {
            fs::create_dir_all(&dir_path)?;
        }

        for file in &self.files {
            let path = dir_path.to_string() + "/" + &file.0;
            let mut new_file = fs::File::create(path)?;
            cat_file::cat_file(&file.1, git_dir_path, &mut new_file)?;
        }

        for subdirs in &self.directories {
            subdirs.create_directories(&dir_path, git_dir_path)?;
        }
        Ok(())
    }

    /// Given a tree, it deletes all the files and directories in the working tree that correspond to the tree.
    /// The tree itself is not modified.
    pub fn delete_directories(&self, parent_dir: &str) -> io::Result<()> {
        let dir_path = if parent_dir.is_empty() {
            parent_dir.to_string() + &self.name
        } else if self.name.is_empty() {
            parent_dir.to_string()
        } else {
            parent_dir.to_string() + "/" + &self.name
        };

        for subdirs in &self.directories {
            subdirs.delete_directories(&dir_path)?;
        }
        for file in &self.files {
            let path = dir_path.to_string() + "/" + &file.0;

            if Path::new(&path).exists() {
                fs::remove_file(path)?;
            }
        }

        if dir_path.is_empty() {
            return Ok(());
        }
        let dir_path_buf = PathBuf::from(&dir_path);
        let is_empty = dir_path_buf.read_dir()?.next().is_none();
        if is_empty {
            fs::remove_dir(dir_path)?;
        }
        Ok(())
    }

    /// Squash the tree into a vector of tuples (file_name, hash). So a file that is in a subtree will have its complete path from the root tree.
    fn squash_tree_into_vec(&self, parent_dir: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        let dir_path = if parent_dir.is_empty() {
            parent_dir.to_string() + &self.name
        } else {
            parent_dir.to_string() + "/" + &self.name
        };
        for file in &self.files {
            let path = dir_path.to_string() + "/" + &file.0;
            result.push((path, file.1.to_string()));
        }
        for subdirs in &self.directories {
            let mut subdirs_vec = subdirs.squash_tree_into_vec(&dir_path);
            result.append(&mut subdirs_vec);
        }
        result
    }

    /// Builds an index file from the tree.
    /// The index file will contain all the files in the tree.
    /// It follows the same format as the index file created by the index module.
    /// That is "path/to/file hash\n"
    /// The index file will be stored in the same directory as the tree.
    pub fn build_index_file_from_tree(
        &self,
        index_path: &str,
        git_dir_path: &str,
        gitignore_path: &str,
    ) -> io::Result<index::Index> {
        let mut index = index::Index::new(index_path, git_dir_path, gitignore_path);
        let entries = self.squash_tree_into_vec("");
        for entry in entries {
            index.add_file(&entry.0, &entry.1)?;
        }
        Ok(index)
    }
}

/// Builds a tree from the index file.
/// Every directory is a tree node, and every file is a leaf.
/// Files that are not listed in a directory in the index file will be part of the root tree.
///
/// The index file must be in the same format as the one created by the index module.
pub fn build_tree_from_index(
    index_path: &str,
    git_dir_path: &str,
    git_ignore_path: &str,
) -> io::Result<Tree> {
    let index = index::Index::load(index_path, git_dir_path, git_ignore_path)?;
    let mut tree = Tree::new("");

    //Iterates over the index struct, adding each file to the tree.
    //It grabs a path, gets the filename (the last part of the path).
    //Then, for every other part of the path, it gets or creates a directory with that name.
    //Starting from the root directory of the tree, it goes down the tree until it reaches the directory where the file should be.
    for (path, hash) in index.iter() {
        let mut path = path.split('/').collect::<Vec<&str>>();
        let file_name = match path.pop() {
            Some(file_name) => file_name,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid path in index file.",
                ))
            }
        };
        let mut current_tree = &mut tree;
        for dir in path {
            current_tree = current_tree.get_or_create_dir(dir);
        }
        current_tree.add_file(file_name, hash);
    }
    Ok(tree)
}

fn map_hexa_tuples_to_bytes(hexa_tuples: Vec<(char, char)>) -> Vec<u8> {
    hexa_tuples
        .iter()
        .map(|(a, b)| {
            let a = a.to_digit(16).unwrap() as u8;
            let b = b.to_digit(16).unwrap() as u8;
            (a << 4) | b
        })
        .collect::<Vec<u8>>()
}

/// Write tree to file in the objects folder.
/// When done, the subtrees are already stored in the objects folder.
/// The result of the function is a tuple of the form (hash, name) corresponding to the root tree.
pub fn write_tree(tree: &Tree, directory: &str) -> io::Result<(String, String)> {
    let mut subtrees: Vec<(String, String)> = Vec::new();

    for sub_dir in &tree.directories {
        let sub_tree = write_tree(sub_dir, directory)?;
        subtrees.push(sub_tree);
    }
    subtrees.sort();

    let tree_content = tree.tree_blobs_to_string_formatted();

    let mut subtrees_vec: Vec<(String, String, Vec<u8>)> = Vec::new();
    for subtree in subtrees {
        let hash_bytes = subtree.0.chars().collect::<Vec<char>>();
        let hash_bytes = hash_bytes
            .chunks(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect::<Vec<(char, char)>>();
        let hash_bytes = map_hexa_tuples_to_bytes(hash_bytes);

        subtrees_vec.push((TREE_MODE.to_string(), subtree.1, hash_bytes));
    }
    let tree_hash = hash_object::store_tree_to_file(tree_content, subtrees_vec, directory)?;
    Ok((tree_hash, tree.name.clone()))
}

/// Wrapper to abstract ourselves from tree naming.
/// Creates a tree looking at the objects folder.
/// When a tree is found in the object file, it loads it and appends it to the current tree.
/// Else, if a blob is found, it adds it to the current tree.
fn _load_tree_from_file(tree_hash: &str, directory: &str, name: &str) -> io::Result<Tree> {
    let tree_content = cat_tree(tree_hash, directory)?;
    let mut tree = Tree::new();
    tree.name = name.to_string();

    for (mode, name, hash) in tree_content {
        let object_type = match mode.as_str() {
            "100644" => "blob",
            "40000 " => "tree", //The space is intentional, fix later.
            _ => "blob",
        };
        match object_type {
            "blob" => tree.add_file(&name, &hash),
            "tree" => {
                _load_tree_from_file(&hash, directory, &name)?;
                tree.directories
                    .push(_load_tree_from_file(&hash, directory, &name)?);
            }
            _ => println!("Invalid tree file."),
        }
    }
    Ok(tree)
}

/// Builds a tree from a tree hash.
/// The tree and its subtrees must be stored in the objects folder, probably by using the write_tree function.
/// The result of the function is a tree with the same structure as the one that was stored.
pub fn load_tree_from_file(tree_hash: &str, directory: &str) -> io::Result<Tree> {
    let tree = _load_tree_from_file(tree_hash, directory, "")?;
    Ok(tree)
}

/// Load a tree (`Tree`) from a specified commit.
///
/// This function takes the hash of a commit and a base directory as input,
/// and loads the tree associated with that commit from the filesystem.
///
/// # Arguments
///
/// * `commit_hash`: The hash of the commit from which to load the tree.
/// * `directory`: The base directory where the content of the commit will be searched.
///
/// # Returns
///
/// An `io::Result<Tree>` that contains the tree loaded from the commit.
///
/// # Errors
///
/// This function can return I/O (`io::Result`) errors if there are issues when reading
/// the content of the commit or loading the tree from the filesystem.
pub fn load_tree_from_commit(commit_hash: &str, directory: &str) -> io::Result<Tree> {
    let commit_content = cat_file::cat_file_return_content(commit_hash, directory)?;
    let splitted_commit_content: Vec<&str> = commit_content.split('\n').collect();
    let first_line_of_commit_file: Vec<&str> = splitted_commit_content[0].split(' ').collect();
    let tree_hash = &first_line_of_commit_file[1];
    let tree = _load_tree_from_file(tree_hash, directory, "")?;
    Ok(tree)
}

pub fn has_tree_changed_since_last_commit(
    new_tree_hash: &str,
    last_commit_hash: &str,
    directory: &str,
) -> bool {
    let commit_content = match cat_file::cat_file_return_content(last_commit_hash, directory) {
        Ok(content) => content,
        Err(_) => return true,
    };
    let splitted_commit_content: Vec<&str> = commit_content.split('\n').collect();
    let first_line_of_commit_file: Vec<&str> = splitted_commit_content[0].split(' ').collect();
    let last_tree_hash = first_line_of_commit_file[1];
    new_tree_hash != last_tree_hash
}

/// Print the contents of a tree to the console with a specified depth of indentation.
///
/// This function recursively prints the files and subdirectories of a tree to the console,
/// adding indentation to represent the directory structure. Each file is displayed with its
/// name and associated hash.
///
/// # Arguments
///
/// * `tree`: A reference to the `Tree` structure to print.
/// * `depth`: The depth of indentation to use for formatting the tree.
///
/// /// # Note
///
/// This function is intended for debugging and visualizing the contents of a `Tree` structure
/// in a human-readable format on the console.
///
pub fn print_tree_console(tree: &Tree, depth: usize) {
    let mut spaces = String::new();
    for _ in 0..depth {
        spaces.push_str("  ");
    }
    for (file_name, hash) in &tree.files {
        println!("{}{} {}", spaces, file_name, hash);
    }
    for dir in &tree.directories {
        println!("{}{}", spaces, dir.name);
        print_tree_console(dir, depth + 1);
    }
}

fn merge_their_tree_into_ours(our_tree: &Tree, their_tree: &Tree, mut new_tree: Tree) -> Tree {
    let their_tree_vec = their_tree.squash_tree_into_vec("");

    for (path, hash) in their_tree_vec {
        let mut path_vec = path.split('/').collect::<Vec<&str>>();
        let filename = match path_vec.pop() {
            Some(filename) => filename,
            None => panic!("Invalid path in index file."),
        };
        let mut current_tree = &mut new_tree;
        for dir in path_vec {
            current_tree = current_tree.get_or_create_dir(dir);
        }
        let our_hash = our_tree.get_hash_from_path(&path);
        match our_hash {
            Some(_) => (),
            None => {
                current_tree.add_file(filename, &hash);
            }
        }
    }
    new_tree
}

/// Merges a file from the current branch with the same file on the other branch if it exists.
fn merge_file(
    path: &str,
    hash: &str,
    their_tree: &Tree,
    current_tree: &mut Tree,
    filename: &str,
    git_dir: &str,
) -> io::Result<()> {
    let their_hash = their_tree.get_hash_from_path(path);
    match their_hash {
        Some(their_hash) => {
            if their_hash == hash {
                current_tree.add_file(filename, hash);
            } else {
                let mut new_file = fs::File::create(path)?;
                let diff = diff::return_object_diff_string(&their_hash, hash, git_dir);

                match diff {
                    Ok(diff) => {
                        new_file.write_all(diff.as_bytes())?;
                        let new_hash = hash_object::store_string_to_file(&diff, git_dir, "blob")?;
                        current_tree.add_file(filename, &new_hash);
                    }
                    Err(_) => {
                        current_tree.add_file(filename, hash);
                    }
                }
            }
        }
        None => {
            current_tree.add_file(filename, hash);
        }
    }
    Ok(())
}

/// Given two trees, it merges them into a new tree.
/// The new tree will have the files of both trees.
/// There are three cases:
/// * If a file is in both trees and has the same hash, it will be added to the new tree.
/// * If a file is in both trees and has different hashes, the diff between the two files will be calculated and added to the new tree.
/// * If a file is in one tree but not in the other, it will be added to the new tree.
///
/// ## Arguments
/// * `our_tree`: The tree of the current branch.
/// * `their_tree`: The tree of the branch we want to merge.
/// * `git_dir`: The path to the git folder.
///
/// ## Errors
/// This function can return I/O (`io::Result`) errors if there are issues when reading
/// the content of the commit or loading the tree from the filesystem.
pub fn merge_trees(our_tree: &Tree, their_tree: &Tree, git_dir: &str) -> io::Result<Tree> {
    let our_tree_vec = our_tree.squash_tree_into_vec("");
    let mut new_tree = Tree::new("");

    for (path, hash) in our_tree_vec {
        let mut path_vec = path.split('/').collect::<Vec<&str>>();
        let filename = match path_vec.pop() {
            Some(filename) => filename,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid path in index file.",
                ))
            }
        };
        let mut current_tree = &mut new_tree;
        for dir in path_vec {
            current_tree = current_tree.get_or_create_dir(dir);
        }
        merge_file(&path, &hash, their_tree, current_tree, filename, git_dir)?;
    }

    let new_tree = merge_their_tree_into_ours(our_tree, their_tree, new_tree);
    Ok(new_tree)
}

//Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        path::Path,
    };
    #[test]
    fn test_get_or_create_dir_2() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        assert!(tree.directories.len() == 1)
    }

    #[test]
    fn test_get_or_create_dir_1() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        tree.get_or_create_dir("root");
        assert!(tree.directories.len() == 1)
    }

    #[test]
    fn test_get_or_create_dir_3() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        tree.get_or_create_dir("name");
        assert!(tree.directories.len() == 2)
    }

    #[test]
    fn test_get_or_create_dir_4() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        tree.get_or_create_dir("root/algo");
        assert!(tree.directories.len() == 2)
    }

    #[test]
    fn test_get_subdir_1() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        let subdir = tree.get_subdir("name");
        assert!(subdir.is_none());
    }

    #[test]
    fn test_get_subdir_2() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        let subdir = tree.get_subdir("root");
        assert!(subdir.is_some());
    }

    #[test]
    fn test_add_file() {
        let mut tree = Tree::new("");
        tree.add_file("root", "059302h2");
        assert!(tree.files.len() == 1);
    }

    #[test]
    fn test_get_depth_1() {
        let tree = Tree::new("");
        assert!(tree.get_depth() == 1);
    }

    #[test]
    fn test_get_depth_2() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        assert!(tree.get_depth() == 2);
    }

    #[test]
    fn test_get_depth_3() {
        let mut tree = Tree::new("");
        tree.get_or_create_dir("root");
        tree.get_or_create_dir("name");
        assert!(tree.get_depth() == 2);
    }

    #[test]
    fn test_get_depth_4() {
        let mut tree = Tree::new("");
        tree.add_file("root", "45739h123c");
        assert!(tree.get_depth() == 1);
    }

    #[test]
    fn test_get_depth_5() {
        let mut tree = Tree::new("");
        let new_tree = tree.get_or_create_dir("root");

        assert!(new_tree.get_depth() == 1 && tree.get_depth() == 2);
    }

    // #[test]
    // fn test_tree_blobs_to_string_formatted() {
    //     let mut tree = Tree::new();
    //     tree.add_file("root", "1");
    //     tree.add_file("test", "2");
    //     let string = tree.tree_blobs_to_string_formatted();
    //     assert_eq!(string, "100644 blob 1 root\n100644 blob 2 test\n");
    // }

    #[test]
    fn test_get_hash_from_path_is_some() {
        let mut tree = Tree::new("");
        tree.add_file("root", "1");
        if let Some(hash) = tree.get_hash_from_path("root") {
            assert_eq!(hash, "1");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_get_hash_from_path_is_none() {
        let mut tree = Tree::new("");
        tree.add_file("root", "1");
        let hash_result = tree.get_hash_from_path("none");
        assert!(hash_result.is_none());
    }

    fn create_if_not_exists(path: &str, is_dir: bool) -> io::Result<()> {
        if !Path::new(path).exists() {
            if is_dir {
                std::fs::create_dir(path)?;
            } else {
                File::create(path)?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_build_tree_from_index() -> io::Result<()> {
        create_if_not_exists("tests/fake_repo", true)?;
        create_if_not_exists("tests/fake_repo/.mgit", true)?;
        create_if_not_exists("tests/fake_repo/.mgit/index_file", false)?;
        create_if_not_exists("tests/fake_repo/.mgitignore", false)?;
        let content = "file1.txt\nfile2.txt\n/.mgit/file3.txt\n";
        let path = "tests/fake_repo/.mgit/index_file";

        let mut index_file = OpenOptions::new().write(true).truncate(true).open(path)?;

        index_file.write_all(content.as_bytes())?;
        let result_tree =
            build_tree_from_index("tests/fake_repo/.mgit/index_file", "tests/fake_repo", "");
        assert!(result_tree.is_ok());
        Ok(())
    }

    #[test]
    fn test_build_tree_from_index_fails() -> io::Result<()> {
        create_if_not_exists("tests/fake_repo", true)?;
        create_if_not_exists("tests/fake_repo/.mgit", true)?;
        create_if_not_exists("tests/fake_repo/.mgit/index_file", false)?;
        create_if_not_exists("tests/fake_repo/.mgitignore", false)?;
        let content = "file1.txt\nfile2.txt\n/.mgit/file3.txt\n";
        let path = "tests/fake_repo/.mgit/index_file";

        let mut index_file = OpenOptions::new().write(true).truncate(true).open(path)?;

        index_file.write_all(content.as_bytes())?;
        let result_tree =
            build_tree_from_index("tests/fake_repo/.mgit/index", "tests/fake_repo", "");
        assert!(result_tree.is_err());
        Ok(())
    }

    // fn create_git_dir(git_dir_path: &str) {
    //     let _ = std::fs::remove_dir_all(git_dir_path);
    //     let _ = std::fs::create_dir(git_dir_path);
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/refs");
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/refs/heads");
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/objects");
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs");
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs/refs");
    //     let _ = std::fs::create_dir(git_dir_path.to_string() + "/logs/refs/heads");
    //     let mut head_file = std::fs::File::create(git_dir_path.to_string() + "/HEAD").unwrap();
    //     head_file
    //         .write_all("ref: refs/heads/main".as_bytes())
    //         .unwrap();

    //     //Create the refs/heads/main file
    //     let mut refs_file =
    //         std::fs::File::create(git_dir_path.to_string() + "/refs/heads/main").unwrap();
    //     refs_file
    //         .write_all("hash_del_commit_anterior".as_bytes())
    //         .unwrap();

    //     //Create the index file
    //     let mut index_file = std::fs::File::create(git_dir_path.to_string() + "/index").unwrap();
    //     index_file.write_all("".as_bytes()).unwrap();
    // }

    // #[test]
    // fn test_write_tree_no_subtrees() {
    //     let git_dir_path = "tests/commit/.mgit_test4";
    //     create_git_dir(git_dir_path);

    //     let content = "hash1 file1.txt\nhash2 file2.txt\nhash3 file3.txt\n";
    //     let path = "tests/commit/.mgit_test4/index";

    //     let mut index_file = OpenOptions::new()
    //         .write(true)
    //         .truncate(true)
    //         .open(path)
    //         .unwrap();

    //     index_file.write_all(content.as_bytes()).unwrap();
    //     let tree = build_tree_from_index(
    //         "tests/commit/.mgit_test4/index",
    //         "tests/commit/.mgit_test4",
    //         "",
    //     )
    //     .unwrap();
    //     let result = write_tree(&tree, "tests/commit/.mgit_test4").unwrap();
    //     let tree_file = cat_file_return_content(&result.0, "tests/commit/.mgit_test4").unwrap();

    //     assert_eq!(
    //         tree_file,
    //         "100644 blob hash1 file1.txt\n100644 blob hash2 file2.txt\n100644 blob hash3 file3.txt\n"
    //     );
    //     let _ = std::fs::remove_dir_all(git_dir_path);
    // }

    // #[test]
    // fn test_write_tree_with_subtrees() {
    //     let git_dir_path = "tests/commit/.mgit_test5";
    //     create_git_dir(git_dir_path);

    //     let content = "hash1 file1.txt\nhash2 file2.txt\nhash3 file3.txt\nhash4 src/file4.txt\n";
    //     let path = "tests/commit/.mgit_test5/index";

    //     let mut index_file = OpenOptions::new()
    //         .write(true)
    //         .truncate(true)
    //         .open(path)
    //         .unwrap();

    //     index_file.write_all(content.as_bytes()).unwrap();
    //     let tree =
    //         build_tree_from_index("tests/commit/.mgit_test5/index", git_dir_path, "").unwrap();
    //     let result = write_tree(&tree, git_dir_path).unwrap();

    //     let tree_file = cat_file_return_content(&result.0, git_dir_path).unwrap();
    //     let tree_file_blob_part = tree_file.split("tree").collect::<Vec<&str>>()[0];
    //     let tree_file_tree_part = tree_file.split("tree").collect::<Vec<&str>>()[1];
    //     let sub_tree_hash = tree_file_tree_part.split(" ").collect::<Vec<&str>>()[1];

    //     let sub_tree_content = cat_file_return_content(&sub_tree_hash, git_dir_path).unwrap();

    //     assert_eq!(
    //         tree_file_blob_part,
    //         "100644 blob hash1 file1.txt\n100644 blob hash2 file2.txt\n100644 blob hash3 file3.txt\n040000 "
    //     );
    //     assert_eq!(sub_tree_content, "100644 blob hash4 file4.txt\n");
    //     let _ = std::fs::remove_dir_all(git_dir_path);
    // }
}
