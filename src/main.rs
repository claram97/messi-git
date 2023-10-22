use std::{fs::File, io::{self}};

use messi::{commit, cat_file, tree_handler::print_tree_console, init::git_init, add};

fn main() {
    
    let repo_result = git_init(".", "main", None);
    
    println!("{:?}", repo_result);

    //Create the index file
    let _ = File::create(".mgit/index").unwrap();

    add::add("tests/hash_object/hash_object_hello.txt", ".mgit/index", ".mgit", None).unwrap();
    add::add("src/cat_file.rs", ".mgit/index", ".mgit", None).unwrap();
    add::add("src/hash_object.rs", ".mgit/index", ".mgit", None).unwrap();
    add::add("src/index.rs", ".mgit/index", ".mgit", None).unwrap();
    add::add("tests/logger_tests.rs", ".mgit/index", ".mgit", None).unwrap();

    let commit_hash = commit::new_commit(".mgit", "probando nuevo commit").unwrap();
    
    cat_file::cat_file(&commit_hash, ".mgit", &mut io::stdout()).unwrap();

    println!("===========================================================================");
    println!("===========================================================================");

    let commit_data = cat_file::cat_file_return_content(&commit_hash, ".mgit").unwrap();
    let tree_hash = commit_data.split_whitespace().nth(1).unwrap();
    let tree = messi::tree_handler::load_tree_from_file(tree_hash, ".mgit").unwrap();
    print_tree_console(&tree, 0);

    let path = "src/index.rs";
    let found_hash = tree.get_hash_from_path(path);
    println!("Hash for path {} is {}", path, found_hash.unwrap());

}
