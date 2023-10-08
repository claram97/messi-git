use std::{path::Path, fs::{self}};


pub fn create_directory(name: &str) {
    let path = Path::new(name);
    if !path.exists() {
        match fs::create_dir(path) {
            Err(why) => panic!("couldn't create directory: {}", why),
            Ok(_) => println!("created directory"),
        }
    }
}

