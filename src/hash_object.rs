use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::utils::get_current_time;
use crate::{configuration::LOGGER_COMMANDS_FILE, logger::Logger};
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

/// Logs the 'git hash-object' command with the specified file path and Git directory.
///
/// This function logs the 'git hash-object' command with the provided file path
/// and Git directory to a file named 'logger_commands.txt'.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the file.
/// * `git_dir_path` - A string slice representing the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_hash_object(path: &str, git_dir_path: &str) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git hash-object': Path '{}', Git Directory '{}', {}",
        path,
        git_dir_path,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

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
    let output_file_dir: String = git_dir_path.to_string() + "/objects/" + &content_hash[..2] + "/";
    create_directory(&output_file_dir)?;
    let output_file_str = output_file_dir + &content_hash[2..];
    compress_content(path, output_file_str.as_str(), "blob")?;
    log_hash_object(path, git_dir_path)?;
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

/// Hashes a byte array using the SHA-1 algorithm and returns the hash as a hexadecimal string.
///
/// This function takes a reference to a byte array (`array`) and calculates its SHA-1 hash.
/// The resulting hash is then formatted as a hexadecimal string and returned.
///
/// # Arguments
///
/// * `array` - A reference to a byte array to be hashed.
///
/// # Returns
///
/// Returns a `String` representing the SHA-1 hash of the input byte array in hexadecimal format.
///
fn hash_byte_array(array: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(array);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Stores a byte array into a file in the Git object database and returns its content hash.
///
/// This function takes a byte array (`content`), a Git directory path (`git_dir_path`),
/// and a file type (`file_type`) as input. It then constructs a Git object file, compresses
/// it using zlib compression, and stores it in the Git object database. The function returns
/// the SHA-1 hash of the content, which serves as its unique identifier.
///
/// # Arguments
///
/// * `content` - The byte array to be stored in the file.
/// * `git_dir_path` - The path to the Git directory.
/// * `file_type` - A string representing the type of the file (e.g., "blob" or "commit").
///
/// # Returns
///
/// Returns a `Result` containing the content hash as a `String` on success, or an `io::Error` on failure.
///
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

    let file = File::create(output_file_str)?;

    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(&complete)?;
    encoder.finish()?;

    Ok(content_hash)
}

/// Stores a tree structure into a file in the Git object database and returns its content hash.
///
/// This function takes two vectors of tuples (`blobs` and `trees`), representing the tree's
/// contents, a Git directory path (`git_dir_path`), and constructs a Git tree object. The
/// function then sorts the entries, calculates the tree's size, creates the object's content,
/// compresses it using zlib compression, and stores it in the Git object database. The function
/// returns the SHA-1 hash of the content, which serves as its unique identifier.
///
/// # Arguments
///
/// * `blobs` - A vector of tuples representing blob entries in the tree. Each tuple contains
///   the file mode, name, and content hash.
/// * `trees` - A vector of tuples representing subtree entries in the tree. Each tuple contains
///   the file mode, name, and tree object hash.
/// * `git_dir_path` - The path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` containing the content hash as a `String` on success, or an `io::Error` on failure.
///
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
    let mut data: Vec<u8> = Vec::new();
    let header = format!("tree {}\0", size);
    data.write_all(header.as_bytes())?;
    for (mode, name, hash) in blobs {
        data.write_all(format!("{} {}\0", mode, name).as_bytes())?;
        data.write_all(&hash)?;
    }
    let tree_hash = hash_byte_array(&data);
    let output_file_dir = git_dir_path.to_string() + "/objects/" + &tree_hash[..2] + "/";
    create_directory(&output_file_dir)?;
    let output_file_str = output_file_dir + &tree_hash[2..];
    compress_tree(data, &output_file_str)?;
    Ok(tree_hash)
}

pub fn get_tree_hash(
    blobs: Vec<(String, String, Vec<u8>)>,
    trees: Vec<(String, String, Vec<u8>)>,
) -> String {
    let mut blobs = blobs;
    blobs.append(&mut trees.clone());
    blobs.sort_by(|a, b| a.1.cmp(&b.1));
    let mut size = 0;
    for (mode, name, hash) in blobs.clone() {
        size += mode.len() + name.len() + hash.len() + 2;
    }
    let mut data: Vec<u8> = Vec::new();
    let header = format!("tree {}\0", size);
    data.write_all(header.as_bytes()).unwrap();
    for (mode, name, hash) in blobs {
        data.write_all(format!("{} {}\0", mode, name).as_bytes())
            .unwrap();
        data.write_all(&hash).unwrap();
    }
    hash_byte_array(&data)
}

/// Compresses a given vector of bytes representing a Git tree object and writes it to a file.
///
/// This function takes a vector of bytes (`tree_vec`) representing the content of a Git tree object
/// and compresses it using zlib compression. The compressed content is then written to the specified
/// output file (`output_file`).
///
/// # Arguments
///
/// * `tree_vec` - A vector of bytes representing the content of a Git tree object.
/// * `output_file` - The path to the output file where the compressed content will be written.
///
/// # Returns
///
/// Returns a `Result` with `Ok(())` on success, or an `io::Error` on failure.
///
fn compress_tree(tree_vec: Vec<u8>, output_file: &str) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(File::create(output_file)?, Compression::default());
    encoder.write_all(&tree_vec)?;
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
