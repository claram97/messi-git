use crate::utils::get_current_time;
use crate::{configuration::LOGGER_COMMANDS_FILE, logger::Logger};
use flate2::bufread::ZlibDecoder;
use std::{
    fs::File,
    io::{self, BufReader, ErrorKind, Read, Write},
};

/// Logs the 'git cat-file' command with the specified parameters.
///
/// This function logs the 'git cat-file' command with the provided parameters to a file named
/// 'logger_commands.txt'.
///
/// # Arguments
///
/// * `hash` - A string slice representing the hash value of the Git object.
/// * `directory` - A string slice representing the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful.
///
pub fn log_cat_file(hash: &str, directory: &str) -> io::Result<()> {
    let log_file_path = LOGGER_COMMANDS_FILE;
    let mut logger = Logger::new(log_file_path)?;

    let full_message = format!(
        "Command 'git cat-file': Hash '{}', Git Dir '{}', {}",
        hash,
        directory,
        get_current_time()
    );
    logger.write_all(full_message.as_bytes())?;
    logger.flush()?;
    Ok(())
}

/// It recieves the complete hash of a file and writes the content of the file to output.
/// output can be anything that implementes Write
/// for example a file or a Vec<u8>
/// For writing to stdout, io::stdout() can be used.
/// If the hash is not valid, it prints "Not a valid hash".
///
/// ## Parameters
/// * `hash` - The complete hash of the file to print.
/// * `directory` - The path to the git directory.
/// * `output` - The output to write the content of the file to. It can be anything that implements Write. For example a file or a Vec<u8>. For writing to stdout, io::stdout() can be used.
pub fn cat_file(hash: &str, directory: &str, output: &mut impl Write) -> io::Result<()> {
    if let Ok(content) = cat_file_return_content(hash, directory) {
        log_cat_file("hash_here", "git_dir_here")?;
        output.write_all(content.as_bytes())
    } else {
        Err(io::Error::new(
            ErrorKind::NotFound,
            "File couldn't be found",
        ))
    }
}

/// It receives the hash of the file to print, the complete hash.
/// If the hash is valid and the file is found, it returns the content of the file as a String.
/// If the hash is not valid, it returns an error.
/// If the hash is valid but the file is not found, it returns an error.
///
/// ## Parameters
/// * `hash` - The complete hash of the file to print.
/// * `directory` - The path to the git directory.
pub fn cat_file_return_content(hash: &str, directory: &str) -> io::Result<String> {
    let file_dir = format!("{}/objects/{}", directory, &hash[..2]);
    let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
    let content = decompress_file(file)?;
    let partes = content.split('\0').nth(1);
    match partes {
        Some(partes) => Ok(partes.to_string()),
        None => Ok(content),
    }
}

/// Extracts and parses the content of a Git tree object.
///
/// This function reads and parses the content of a Git tree object identified by its `hash` from the local
/// Git repository located in the directory specified by `directory`. It returns a list of tuples, each containing
/// the mode, name, and hash associated with entries in the tree object.
///
/// The function performs the following steps:
/// 1. Opens and decompresses the Git object file.
/// 2. Extracts the tree object's header.
/// 3. Parses the entries within the tree, including their mode, name, and hash.
///
/// # Arguments
///
/// * `hash`: The hash identifier of the Git tree object to extract and parse.
/// * `directory`: The path to the local directory containing the Git repository.
///
/// # Returns
///
/// Returns a `Result` containing a vector of tuples, each with three elements: mode, name, and hash of entries
/// in the tree object. In case of success, the result is wrapped in an `io::Result<Vec<(String, String, String)>>`.
///
pub fn cat_tree(hash: &str, directory: &str) -> io::Result<Vec<(String, String, String)>> {
    let file_dir = format!("{}/objects/{}", directory, &hash[..2]);
    let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;

    let content = decompress_into_bytes(file)?;

    let header_len = match content.iter().position(|&x| x == 0) {
        Some(pos) => pos,
        None => {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Null character not found",
            ))
        }
    };
    let header = content.split_at(header_len);
    let content = header.1[1..].to_vec();

    //Entry del tree: <modo> <nombre>\0<hash>

    let mut results = vec![];
    let mut r = BufReader::new(content.as_slice());

    let mut bytes_read = 0;
    while bytes_read < content.len() {
        // mientras no haya leido todo el contenido
        let mut mode: [u8; 6] = [0, 0, 0, 0, 0, 0]; // leo los 6 bytes del modo
        r.read_exact(&mut mode)?;
        bytes_read += 6;
        let mut name: Vec<u8> = vec![]; // lo prixmo es el nombre hasta el \0
        if mode[0] != 52 {
            r.read_exact(&mut [0])?; // salteo el espacio
            bytes_read += 1;
        }
        let mut buf: [u8; 1] = [0];
        loop {
            r.read_exact(&mut buf)?;
            bytes_read += 1;
            if buf[0] == 0 {
                // si es el \0 termino
                break;
            }
            name.push(buf[0]);
        }

        let mut hash = [0; 20]; // leo los 20 bytes del hash
        r.read_exact(&mut hash)?;
        bytes_read += 20;

        let mode = String::from_utf8(mode.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?; // lo paso a string
        let hash: Vec<String> = hash.iter().map(|byte| format!("{:02x}", byte)).collect(); // convierto los bytes del hash a string
        let hash = hash.concat().to_string();
        let name = String::from_utf8(name.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        results.push((mode, name, hash)); // agrego el resultado y vuelvo a empezar
    }
    Ok(results)
}

/// Decompresses the content of a Zlib-compressed file into a byte vector.
///
/// This function decompresses the content of a Zlib-compressed file provided as an input `File` into a byte
/// vector. It returns the decompressed content as a vector of unsigned 8-bit integers (`u8`).
///
/// # Arguments
///
/// * `file`: A `File` representing the Zlib-compressed file to decompress.
///
/// # Returns
///
/// Returns a `Result` containing the decompressed content as a vector of `u8`. In case of success, the result
/// is wrapped in an `io::Result<Vec<u8>>`.
///
fn decompress_into_bytes(file: File) -> io::Result<Vec<u8>> {
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = Vec::new();
    decompressor.read_to_end(&mut decompressed_content)?;
    Ok(decompressed_content)
}

/// Decompresses a given file.
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
        let content = cat_file_return_content(hash, "tests/cat_file").unwrap();
        assert_eq!(content, "Hello World!");
    }

    #[test]
    fn test_decompress_file() {
        let file =
            std::fs::File::open("tests/cat_file/objects/c5/7eff55ebc0c54973903af5f72bac72762cf4f4")
                .unwrap();
        let content = super::decompress_file(file).unwrap();
        assert_eq!(content, "blob 12\0Hello World!");
    }
}
