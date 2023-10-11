use std::{io::{self, BufReader, Read}, fs::File, env};

use flate2::bufread::ZlibDecoder;

/// Returns the path to the .git directory if it exists in the current directory or any of its parents.
/// Returns None if the .git directory is not found.
fn find_git_directory() -> Option<String> {
    if let Ok(current_dir) = env::current_dir() {
        let mut current_dir = current_dir;

        loop {
            let git_dir = current_dir.join(".git");
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

/// Prints the content of a file given its hash
/// It recieves the hash of the file to print, the complete hash.
/// It returns the content of the file as a String.
pub fn cat_file(hash: &str) -> io::Result<String> {
    if let Some(git_dir) = find_git_directory() {
        let file_dir = format!("{}/objects/{}", git_dir, &hash[..2]);
        let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
        let content = decompress_file(file)?;
        let partes = content.split('\0').nth(1);
        match partes {
            Some(partes) => {
                println!("{}", partes);
                Ok(partes.to_string())
            },
            None => {
                println!("{}", content);
                Ok(content)
            }
        }
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Not a git repository"))
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
        let content = cat_file(hash).unwrap();
        assert_eq!(content, "Hello World!");
    }
    
    // #[test]
    // fn test_decompress_file() {
    //     let file = std::fs::File::open(".git/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4").unwrap();
    //     let content = super::decompress_file(file).unwrap();
    //     assert_eq!(content, "blob 12\0Hello World!");
    // }
}