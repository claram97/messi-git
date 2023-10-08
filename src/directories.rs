use std::{path::Path, fs, io};

use crate::hash_object;

pub fn create_directory(name: &str) {
    let path = Path::new(name);
    if !path.exists() {
        match fs::create_dir(path) {
            Err(why) => panic!("couldn't create directory: {}", why),
            Ok(_) => println!("created directory"),
        }
    }
}

pub fn store_file(path: &str) -> io::Result<String> {
    let content_hash = hash_object::hash_file_content(path)?;
    let file_path = Path::new("objects").join(&content_hash);
    fs::copy(path, file_path)?;
    Ok(content_hash)
}