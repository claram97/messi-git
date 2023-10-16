use std::io::{self};

use crate::{index::{self}, hash_object};

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

}

// From an index file, loading it in an Index struct it builds a tree.
// The result is a tree of trees, where each tree is a directory and each leaf is a file.
// The tree is built recursively.
// The tree is built using the index file.
// The index file is a file that contains the hash of each file in the staging area and the path to the file.
pub fn build_tree(index_path: &str) -> Tree {
    let git_dir_path = String::from(".mgit");
    let index = index::Index::load(&index_path, &git_dir_path).unwrap();
    let mut tree = Tree::new();

    //Iterates over the index struct, adding each file to the tree.
    //It grabs a path, gets the filename (the last part of the path).
    //Then, for every other part of the path, it gets or creates a directory with that name.
    //Starting from the root directory of the tree, it goes down the tree until it reaches the directory where the file should be.
    for (path, hash) in index.iter() {
        let mut path = path.split('/').collect::<Vec<&str>>();
        let file_name = path.pop().unwrap();
        let mut current_tree = &mut tree;
        for dir in path {
            current_tree = current_tree.get_or_create_dir(dir);
        }
        current_tree.add_file(file_name, hash);
    }
    tree
}

//Write tree to file in the objects folder.
//When done, the subtrees should be hashed, compressed and stored.
//The result is the hash of the tree.
pub fn write_tree(tree: &Tree, directory: &str) -> io::Result<(String, String)>{
    //Vec of hash, name tuples
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