use std::io::{self};

use crate::{index::{self}, hash_object, cat_file::cat_file_return_content};

//Tree structure
//files is a vector of tuples (file_name, hash)
#[derive(Debug)]
pub struct Tree {
    pub name: String,
    pub files: Vec<(String, String)>,
    pub directories: Vec<Tree>,
}

impl Tree {
    fn new() -> Self {
        Self {
            name: String::from("root"),
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    /// Gets a name, if the directory with that name exists, returns a mutable reference to it.
    /// If it does not exist, creates a new directory with that name and returns a mutable reference to it.
    /// The directory is added to the directories vector.
    fn get_or_create_dir(&mut self, name: &str) -> &mut Tree {
        for (i, dir) in self.directories.iter().enumerate() {
            if dir.name == name {
                return &mut self.directories[i];
            }
        }
        let mut new_dir = Tree::new();
        new_dir.name = name.to_string();
        self.directories.push(new_dir);
        let last_dir_index = self.directories.len() - 1;
        &mut self.directories[last_dir_index]
    }

    //Get a subdir from a tree
    fn get_subdir(&self, name: &str) -> Option<&Tree> {
        for dir in &self.directories {
            if dir.name == name {
                return Some(dir);
            }
        }
        None
    }


    fn add_file(&mut self, name: &str, hash: &str) {
        //self.files.push((name.to_string(), hash.to_string()));
        
        //Insert ordered using binary search so that the files are ordered alphabetically.
        //And the resulting trees are deterministic.
        let mut start = 0;
        let mut end = self.files.len();
        let mut middle = (start + end) / 2;
        while start < end {
            if self.files[middle].0 < name.to_string() {
                start = middle + 1;
            } else {
                end = middle;
            }
            middle = (start + end) / 2;
        }
        self.files.insert(middle, (name.to_string(), hash.to_string()));

        //Might be better to use a binary heap.
    }

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

    pub fn tree_blobs_to_string_formatted(&self) -> String {
        let mut result = String::new();
        for (file_name, hash) in &self.files {
            result.push_str(format!("blob {} {}\n", hash, file_name).as_str());
        }
        result
    }

    //Given a path, this function should return the hash correspondent to it in the tree.
    //If the path does not exist, it returns None.
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
}

// From an index file, loading it in an Index struct it builds a tree.
// The result is a tree of trees, where each tree is a directory and each leaf is a file.
// The tree is built recursively.
// The tree is built using the index file.
// The index file is a file that contains the hash of each file in the staging area and the path to the file.
pub fn build_tree_from_index(index_path: &str, git_dir_path: &str) -> io::Result<Tree> {
    let index = index::Index::load(&index_path, &git_dir_path).unwrap();
    let mut tree = Tree::new();

    //Iterates over the index struct, adding each file to the tree.
    //It grabs a path, gets the filename (the last part of the path).
    //Then, for every other part of the path, it gets or creates a directory with that name.
    //Starting from the root directory of the tree, it goes down the tree until it reaches the directory where the file should be.
    for (path, hash) in index.iter() {
        let mut path = path.split('/').collect::<Vec<&str>>();
        let file_name = match path.pop() {
            Some(file_name) => file_name,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Invalid path in index file.")),
        };
        let mut current_tree = &mut tree;
        for dir in path {
            current_tree = current_tree.get_or_create_dir(dir);
        }
        current_tree.add_file(file_name, hash);
    }
    Ok(tree)
}

//Write tree to file in the objects folder.
//When done, the subtrees should be hashed, compressed and stored.
//The result of the function is a tuple of the form (hash, name).
pub fn write_tree(tree: &Tree, directory: &str) -> io::Result<(String, String)>{
    //Vec of (hash, name) tuples
    let mut subtrees: Vec<(String, String)> = Vec::new();

    //Traverse the tree, hashing and compressing each subtree.
    for sub_dir in &tree.directories {
        let sub_tree = write_tree(&sub_dir, directory)?;
        subtrees.push(sub_tree);
    }
    
    //Sort the subtrees by name so the resulting tree is deterministic.
    subtrees.sort();
    
    //When all of the subtrees have been traversed, i can write "myself"
    let tree_content = tree.tree_blobs_to_string_formatted();
    let mut subtrees_formatted: String = "".to_owned();
    for subtree in subtrees {
        subtrees_formatted.push_str(format!("tree {} {}\n", subtree.0, subtree.1).as_str());
    }

    let tree_content = tree_content + &subtrees_formatted;
    println!("tree: {}\ntree content:\n{}", tree.name, tree_content);
    //Store and hash the tree content.
    let tree_hash = hash_object::store_string_to_file(&tree_content, directory ,"tree")?;
    Ok((tree_hash, tree.name.clone()))
}


fn _load_tree_from_file(tree_hash: &str, directory: &str, name: &str) -> io::Result<Tree> {
    let tree_content = cat_file_return_content(tree_hash, directory)?;
    let mut tree = Tree::new();
    tree.name = name.to_string();
    let lines = tree_content.lines();

    for line in lines {
        let line = line.split(' ').collect::<Vec<&str>>();
        let object_type = line[0];
        let hash = line[1];
        let name = line[2];
        match object_type {
            "blob" => tree.add_file(name, hash),
            "tree" => {
                _load_tree_from_file(hash, directory, name)?;
                tree.directories.push(_load_tree_from_file(hash, directory, name)?);
            },
            _ => println!("Invalid tree file."),
        }
    }
    Ok(tree)
}

pub fn load_tree_from_file (tree_hash: &str, directory: &str) -> io::Result<Tree> {
    let tree = _load_tree_from_file(tree_hash, directory, "root")?;
    Ok(tree)
}

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
        print_tree_console(&dir, depth + 1);
    }
}

//Tests

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_get_hash_from_path() {
        
    // }
}