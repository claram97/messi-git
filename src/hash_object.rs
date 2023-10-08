use std::{io, fs::File};

use sha1::{Digest, Sha1};
use flate2::{write::ZlibEncoder, Compression};

pub fn hash_string(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn hash_file_content(path: &str) -> io::Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(hash_string(&content))
}

pub fn store_file(path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path)?;
    let output_file_str = "objects/".to_string() + &content_hash;
    compress_content(path, output_file_str.as_str())?;
    Ok(content_hash)
}

fn compress_content(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let mut encoder = ZlibEncoder::new(output_file, Compression::default());

    std::io::copy(&mut input_file, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}
