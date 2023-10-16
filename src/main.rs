use std::{fs::File, io::Write};

use messi::{hash_object, tree_handler};

fn main() {
    let hash1 = hash_object::store_file(
        "tests/hash_object/hash_object_hello.txt",
        ".mgit",
    ).unwrap();

    let hash2 = hash_object::store_file(
        "src/cat_file.rs",
        ".mgit",
    ).unwrap();

    let hash3 = hash_object::store_file(
        "src/hash_object.rs",
        ".mgit",
    ).unwrap();

    let hash4 = hash_object::store_file(
        "src/index.rs",
        ".mgit",
    ).unwrap();

    let hash5 = hash_object::store_file(
        "tests/logger_tests.rs",
        ".mgit",
    ).unwrap();

    //Create an index file
    let mut index = File::create(".mgit/index").unwrap();

    //Write the hashes to the index file with its path
    index.write_all(format!("{} {}\n", hash1, "tests/hash_object/hash_object_hello.txt").as_bytes()).unwrap();
    index.write_all(format!("{} {}\n", hash2, "src/cat_file.rs").as_bytes()).unwrap();
    index.write_all(format!("{} {}\n", hash3, "src/hash_object.rs").as_bytes()).unwrap();
    index.write_all(format!("{} {}\n", hash4, "src/index.rs").as_bytes()).unwrap();
    index.write_all(format!("{} {}\n", hash5, "tests/logger_tests.rs").as_bytes()).unwrap();

    //Create a tree file
    let tree = tree_handler::build_tree(".mgit/index");
    //println!("{:#?}", tree);
    let _ = tree_handler::write_tree(&tree, ".mgit");
}
