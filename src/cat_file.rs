use std::{
    fs::File,
    io::{self, BufReader, ErrorKind, Read, Write},
};

use flate2::bufread::ZlibDecoder;

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
