use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

/// Returns the sha1 hash of the given content.
/// It does not add any type information to the content.
/// Do not use for git objects search. Use hash_file_content instead !!!!!
fn hash_string(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Returns the sha1 hash of the given file content adding the type information.
/// The type information is added as a header to the content.
/// The header is of the form: <type> <size>\0
/// Use this function when searching for a file git object.
/// This function does not return the path to the object in the objects folder, it returns the complete string.
/// **It does not store the file**.
/// ## Parameters
/// * `path` - The path to the file.
/// * `file_type` - The type of the file. It is used to create the header.
///
pub fn hash_file_content(path: &str, file_type: &str) -> io::Result<String> {
    let content = std::fs::read_to_string(path)?;
    let header = format!("{file_type} {}\0", content.len());
    let complete = header + &content;
    Ok(hash_string(&complete))
}

/// Returns the path to the file object in the objects folder.
/// The path is of the form: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The result is the place where the object corresponding to the given file is stored.
///
/// ## Parameters
/// * `path` - The path to the file.
/// * `directory` - The path to the git directory.
///
pub fn get_file_object_path(path: &str, git_dir_path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path, "blob")?;
    let output_file_dir = git_dir_path.to_string() + "/objects/" + &content_hash[..2] + "/";
    let output_file_str = output_file_dir + &content_hash[2..];
    Ok(output_file_str)
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

/// Stores the file at the given path in the objects folder of the given directory.
/// Directory must be the path to the git folder.
/// Returns the hash of the file content.
///
/// Stores the file in the path: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The file is compressed using zlib.
///
/// The content is prepended with the header: blob <size>\0. The size is the size of the content.
///
/// If the directory is not a git directory, it returns an error.
/// If the directory does not have an objects folder, it returns an error.
/// If the file does not exist, it returns an error.
/// If the file is already stored, it stores it again.
///
/// ## Parameters
/// * `path` - The path to the file.
/// * `directory` - The path to the git directory.
///
///
pub fn store_file(path: &str, git_dir_path: &str) -> io::Result<String> {
    let content_hash = hash_file_content(path, "blob")?;
    let output_file_dir = git_dir_path.to_string() + "/objects/" + &content_hash[..2] + "/";
    create_directory(&output_file_dir)?;
    let output_file_str = output_file_dir + &content_hash[2..];
    compress_content(path, output_file_str.as_str(), "blob")?;
    Ok(content_hash)
}

/// Stores the given content in the objects folder of the given directory.
/// Directory must be the path to the git folder.
/// Returns the hash of the content or an error if the directory is not a git directory or if the directory does not have an objects folder.
///
/// /// If the directory is not a git directory, it returns an error.
/// If the directory does not have an objects folder, it returns an error.
/// If the file does not exist, it returns an error.
/// If the file is already stored, it stores it again.
///
/// Stores the file in the path: objects/<first 2 characters of hash>/<remaining characters of hash>
/// The file is compressed using zlib.
///
/// The content is prepended with the header: <type> <size>\0. The size is the size of the content.
///
/// ## Parameters
/// * `content` - The content to store.
/// * `directory` - The path to the git directory.
/// * `file_type` - The type of the file. It is used to create the header.
///
pub fn store_string_to_file(
    content: &str,
    git_dir_path: &str,
    file_type: &str,
) -> io::Result<String> {
    let content_hash = hash_string(&format!("{} {}\0{}", file_type, content.len(), content));

    let output_file_dir = git_dir_path.to_string() + "/objects/" + &content_hash[..2] + "/";
    create_directory(&output_file_dir)?;
    let output_file_str = output_file_dir + &content_hash[2..];

    let tmp_file_path = output_file_str.clone() + "tmp";
    let mut tmp_file = File::create(&tmp_file_path)?;
    tmp_file.write_all(content.as_bytes())?;

    compress_content(&tmp_file_path, output_file_str.as_str(), file_type)?;
    fs::remove_file(tmp_file_path)?;
    Ok(content_hash)
}

fn hash_byte_array(array: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&array);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn store_bytes_array_to_file(
    content: Vec<u8>,
    git_dir_path: &str,
    file_type: &str,
) -> io::Result<String> {
    let header = format!("{file_type} {}\0", content.len());
    let header = header.as_bytes();
    let complete = [header, &content].concat();
    let content_hash = hash_byte_array(&complete);

    //Create the directory where the file will be stored
    let output_file_dir = git_dir_path.to_string() + "/objects/" + &content_hash[..2] + "/";
    create_directory(&output_file_dir)?;

    //Create the path where the file will be stored
    let output_file_str = output_file_dir + &content_hash[2..];

    let file = File::create(&output_file_str)?;

    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(&complete)?;
    encoder.finish()?;

    Ok(content_hash)
}

fn hash_tree_file(tree_file: &str) -> io::Result<String> {
    let content = fs::read(tree_file)?;
    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn store_tree_to_file(
    blobs: Vec<(String, String, Vec<u8>)>,
    trees: Vec<(String, String, Vec<u8>)>,
    git_dir_path: &str,
) -> io::Result<String> {
    let mut blobs = blobs;
    blobs.append(&mut trees.clone());

    blobs.sort_by(|a, b| a.1.cmp(&b.1));

    let mut size = 0;
    for (mode, name, hash) in blobs.clone() {
        size += mode.len() + name.len() + hash.len() + 2;
    }
    let header = format!("tree {}\0", size);
    let mut file = File::create("tree.tmp")?;
    file.write_all(header.as_bytes())?;

    for (mode, name, hash) in blobs {
        file.write_all(format!("{} {}\0", mode, name).as_bytes())?;
        file.write_all(&hash)?;
    }
    drop(file);
    let tree_hash = hash_tree_file("tree.tmp")?;
    let output_file_dir = git_dir_path.to_string() + "/objects/" + &tree_hash[..2] + "/";
    create_directory(&output_file_dir)?;
    let output_file_str = output_file_dir + &tree_hash[2..];
    compress_tree("tree.tmp", &output_file_str)?;
    fs::remove_file("tree.tmp")?;

    Ok(tree_hash)
}

fn compress_tree(tree_file: &str, output_file: &str) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(File::create(output_file)?, Compression::default());
    let tree_file = std::fs::read(tree_file)?;

    encoder.write_all(&tree_file)?;
    encoder.finish()?;
    Ok(())
}

/// Compresses the content of the file at the given input path and stores it in the file at the given output path.
/// The content is compressed using zlib.
/// The content is prepended with the header: blob <size>\0. The size is the size of the content.
fn compress_content(input_path: &str, output_path: &str, file_type: &str) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    let mut encoder = ZlibEncoder::new(output_file, Compression::default());

    let mut content = std::fs::read_to_string(input_path)?;
    let header = format!("{file_type} {}\0", content.len());
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
        let hash = hash_file_content("tests/hash_object/hash_object_hello.txt", "blob").unwrap();
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }

    #[test]
    fn test_store_file_hash() {
        let hash = store_file(
            "tests/hash_object/hash_object_hello.txt",
            "tests/hash_object",
        )
        .unwrap();
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }

    #[test]
    fn test_store_file_content() {
        // Delete the previous file if it exists
        let _ = std::fs::remove_file(
            "tests/hash_object/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4",
        );

        let _hash = store_file(
            "tests/hash_object/hash_object_hello.txt",
            "tests/hash_object",
        )
        .unwrap();
        let content =
            std::fs::read("tests/hash_object/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4")
                .unwrap();
        let mut decoder = ZlibDecoder::new(&content[..]);
        let mut decoded_content = String::new();
        decoder.read_to_string(&mut decoded_content).unwrap();
        assert_eq!(decoded_content, "blob 12\0Hello World!");
    }

    #[test]
    fn store_file_does_not_exist() {
        let result = store_file("tests/hash_object/does_not_exist.txt", "tests/hash_object");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_dir_doe_not_exist() {
        let result = store_file(
            "tests/hash_object/hash_object_hello.txt",
            "tests/does_not_exist",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_no_objects_folder() {
        let result = store_file(
            "tests/hash_object/hash_object_hello.txt",
            "tests/hash_object/no_objects_folder",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_file_content_absolute_path() {
        let curr_env_dir = std::env::current_dir().unwrap();
        let binding = curr_env_dir.join("tests/hash_object/hash_object_hello.txt");
        let absolute_path = binding.to_str().unwrap();
        let hash = hash_file_content(absolute_path, "blob").unwrap();
        assert_eq!(hash, "c57eff55ebc0c54973903af5f72bac72762cf4f4");
    }
}
