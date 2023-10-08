use std::io;

use sha1::{Digest, Sha1};

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