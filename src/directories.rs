use std::{path::Path, fs::{self, File}, io::{self}};

use flate2::{write::ZlibEncoder, Compression};

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

fn compress_content(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let mut encoder = ZlibEncoder::new(output_file, Compression::default());

    std::io::copy(&mut input_file, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

pub fn store_file(path: &str) -> io::Result<String> {
    let content_hash = hash_object::hash_file_content(path)?;
    let output_file_str = "objects/".to_string() + &content_hash;
    compress_content(path, output_file_str.as_str())?;
    Ok(content_hash)
}