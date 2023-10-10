use std::{io::{self, Write}, fs::{File, self}, path::Path};

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
    let header = format!("blob {}\0", content.len());
    let complete = header + &content;
    Ok(hash_string(&complete))
}

fn create_directory(name: &str) {
    let path = Path::new(name);
    if !path.exists() {
        match fs::create_dir(path) {
            Err(why) => panic!("couldn't create directory: {}", why),
            Ok(_) => println!("created directory"),
        }
    }
}

/// Stores the file at the given path in the objects directory.
/// Returns the hash of the file content.
/// 
/// Stores the file in the path: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The file is compressed using zlib.
pub fn store_file(path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path)?;
    let output_file_dir = "objects/".to_string() + &content_hash[..2] + "/";
    create_directory(&output_file_dir);
    let output_file_str = output_file_dir + &content_hash[2..];
    compress_content(path, output_file_str.as_str())?;
    Ok(content_hash)
}

fn compress_content(input_path: &str, output_path: &str) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    let mut encoder = ZlibEncoder::new(output_file, Compression::default());

    let mut content = std::fs::read_to_string(input_path)?;
    let header = format!("blob {}\0", content.len());
    content.insert_str(0, &header);

    encoder.write_all(content.as_bytes())?;

    encoder.finish()?;
    Ok(())
}
