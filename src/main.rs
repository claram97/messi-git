use std::{
    fs::File,
    io::{self, Write},
};

use messi::{cat_file, commit, hash_object, tree_handler::print_tree_console};

fn main() {

    messi::init::git_init(".", "main", None).unwrap();

    let hash1 =
        hash_object::store_file("tests/hash_object/hash_object_hello.txt", ".mgit").unwrap();

    let hash2 = hash_object::store_file("src/cat_file.rs", ".mgit").unwrap();

    let hash3 = hash_object::store_file("src/hash_object.rs", ".mgit").unwrap();

    let hash4 = hash_object::store_file("src/index.rs", ".mgit").unwrap();

    let hash5 = hash_object::store_file("tests/logger_tests.rs", ".mgit").unwrap();

    //Create an index file
    let mut index = File::create(".mgit/index").unwrap();

    //Write the hashes to the index file with its path
    index
        .write_all(format!("{} {}\n", hash1, "tests/hash_object/hash_object_hello.txt").as_bytes())
        .unwrap();
    index
        .write_all(format!("{} {}\n", hash2, "src/cat_file.rs").as_bytes())
        .unwrap();
    index
        .write_all(format!("{} {}\n", hash3, "src/hash_object.rs").as_bytes())
        .unwrap();
    index
        .write_all(format!("{} {}\n", hash4, "src/index.rs").as_bytes())
        .unwrap();
    index
        .write_all(format!("{} {}\n", hash5, "tests/logger_tests.rs").as_bytes())
        .unwrap();

    //Create a commit file
    let commit_hash = commit::new_commit(".mgit", "probando nuevo commit").unwrap();

    cat_file::cat_file(&commit_hash, ".mgit", &mut io::stdout()).unwrap();

    println!("===========================================================================");
    println!("===========================================================================");

    let tree = messi::tree_handler::load_tree_from_file(
        "c78a81f71e0a1110498ce3b86e53dd4872d3efe0",
        ".mgit",
    )
    .unwrap();
    print_tree_console(&tree, tree.get_depth());

    let path = "src/index.rs";
    let found_hash = tree.get_hash_from_path(path);
    println!("Hash for path {} is {}", path, found_hash.unwrap());
}
