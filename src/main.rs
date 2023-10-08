use std::io;

use crate::cat_file::cat_file;

mod hash_object;
mod directories;
pub mod cat_file;

fn prueba() -> io::Result<String> {
    directories::create_directory("objects");
    println!("Hello");
    let mut hashed = hash_object::hash_string("Hello");
    println!("{}", hashed);
    println!("Hello!");
    hashed = hash_object::hash_string("Hello!");
    println!("{}", hashed);

    let main_hash = hash_object::store_file("src/main.rs")?;
    let hash_object_hash = hash_object::store_file("src/hash_object.rs")?;
    let hash_txt = hash_object::store_file("src/hola.txt")?;

    cat_file(&main_hash)?;
    println!("-------------------");
    cat_file(&hash_object_hash)?;
    println!("-------------------");
    cat_file(&hash_txt)?;
    Ok("Ok".to_string())
}

fn main() {
    let _ = prueba();
}
