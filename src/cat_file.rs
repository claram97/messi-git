use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
};

use flate2::bufread::ZlibDecoder;

const GIT_DIR: &str = ".mgit";

/// Returns the path to the .git directory if it exists in the current directory or any of its parents.
/// Returns None if the .git directory is not found.
fn find_git_directory() -> Option<String> {
    if let Ok(current_dir) = env::current_dir() {
        let mut current_dir = current_dir;

        loop {
            let git_dir = current_dir.join(GIT_DIR);
            if git_dir.exists() && git_dir.is_dir() {
                return Some(git_dir.display().to_string());
            }

            if !current_dir.pop() {
                break; // Reached the root directory, the git directory was not found.
            }
        }
    }
    None
}

/// It recieves the hash of the file to print, the complete hash.
/// It prints to stdout the content of the file.
/// If the hash is not valid, it prints "Not a valid hash".
pub fn cat_file(hash: &str) {
    if let Ok(content) = cat_file_return_content(hash) {
        println!("{}", content);
    } else {
        println!("Not a valid hash");
    }
}

/// It recieves the hash of the file to print, the complete hash.
/// If the hash is valid and the file is found, it returns the content of the file as a String.
/// If the hash is not valid, it returns an error.
/// If the hash is valid but the file is not found, it returns an error.
pub fn cat_file_return_content(hash: &str) -> io::Result<String> {
    if let Some(git_dir) = find_git_directory() {
        let file_dir = format!("{}/objects/{}", git_dir, &hash[..2]);
        let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
        let content = decompress_file(file)?;
        let partes = content.split('\0').nth(1);
        match partes {
            Some(partes) => Ok(partes.to_string()),
            None => Ok(content),
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Not a git repository",
        ))
    }
}

/// Decompresses a file given a File
/// Using the flate2 library, it decompresses the file and returns its content as a String.
fn decompress_file(file: File) -> io::Result<String> {
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = String::new();
    decompressor.read_to_string(&mut decompressed_content)?;
    Ok(decompressed_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_file() {
        let hash = "c57eff55ebc0c54973903af5f72bac72762cf4f4";
        let content = cat_file_return_content(hash).unwrap();
        assert_eq!(content, "Hello World!");
    }

    #[test]
    fn test_decompress_file() {
        let file =
            std::fs::File::open(".git/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4").unwrap();
        let content = super::decompress_file(file).unwrap();
        assert_eq!(content, "blob 12\0Hello World!");
    }
}
