use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

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

/// Returns the sha1 hash of the given content.
/// It does not add any type information to the content.
/// Do not use for git objects search. Use hash_file_content instead !!!!!
pub fn hash_string(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Returns the sha1 hash of the given file content adding the type information.
/// The type information is added as a header to the content.
/// The header is of the form: <type> <size>\0
/// Use this function when searching for a file git object.
/// This function does not return the path to the object in the objects folder.
pub fn hash_file_content(path: &str) -> io::Result<String> {
    let content = std::fs::read_to_string(path)?;
    let header = format!("blob {}\0", content.len());
    let complete = header + &content;
    Ok(hash_string(&complete))
}

/// Returns the path to the file object in the objects folder.
/// The path is of the form: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The result is the place where the object corresponding to the given file is stored.
pub fn get_file_object_path(path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path)?;
    if let Some(git_dir) = find_git_directory() {
        let output_file_dir = git_dir + "/objects/" + &content_hash[..2] + "/";
        let output_file_str = output_file_dir + &content_hash[2..];
        Ok(output_file_str)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Git directory not found",
        ))
    }
}

/// Creates a directory with the given name if it does not exist.
fn create_directory(name: &str) -> io::Result<()> {
    let path = Path::new(name);
    if !path.exists() {
        match fs::create_dir(path) {
            Err(why) => Err(why),
            Ok(_) => Ok(()),
        }
    } else {
        Ok(())
    }
}

/// Stores the file at the given path in the objects directory.
/// Returns the hash of the file content.
///
/// Stores the file in the path: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The file is compressed using zlib.
pub fn store_file(path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path)?;
    if let Some(git_dir) = find_git_directory() {
        let output_file_dir = git_dir + "/objects/" + &content_hash[..2] + "/";
        create_directory(&output_file_dir)?;
        let output_file_str = output_file_dir + &content_hash[2..];
        compress_content(path, output_file_str.as_str())?;
        Ok(content_hash)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Git directory not found",
        ))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    #[test]
    fn test_hash_string_without_type() {
        let content = "Hello World!";
        let hash = hash_string(content);
        assert_eq!(hash, "2ef7bde608ce5404e97d5f042f95f89f1c232871");
    }

    // To test this in console: echo -ne "blob 12\0Hello World!" | openssl sha1
    #[test]
    fn test_hash_string_with_type() {
        let content = "blob 12\0Hello World!";
        let hash = hash_string(content);
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }

    #[test]
    fn test_hash_file_content() {
        let hash = hash_file_content("tests/tests_files/hash_object_hello.txt").unwrap();
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }

    #[test]
    fn test_store_file_hash() {
        let hash = store_file("tests/tests_files/hash_object_hello.txt").unwrap();
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }

    #[test]
    fn test_store_file_content() {
        let _hash = store_file("tests/tests_files/hash_object_hello.txt").unwrap();
        let content =
            std::fs::read(".git/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4").unwrap();
        let mut decoder = ZlibDecoder::new(&content[..]);
        let mut decoded_content = String::new();
        decoder.read_to_string(&mut decoded_content).unwrap();
        assert_eq!(decoded_content, "blob 12\0Hello World!");
    }

    #[test]
    fn store_file_does_not_exist() {
        let result = store_file("tests/tests_files/does_not_exist.txt");
        assert!(result.is_err());
    }
}
