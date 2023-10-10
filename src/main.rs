use messi::{directories::create_directory, hash_object};

fn main() {
    println!("Hello, World!");

    create_directory("objects");

    let result = hash_object::store_file("src/hash_object.rs");
    println!("{:?}", result)
}
