use std::{
    fs::File,
    io::{self, BufRead, BufReader, Error, Read},
    str::from_utf8,
};

use flate2::bufread::ZlibDecoder;

use super::object_type::ObjectType;

/// A packfile entry.
/// It contains the object type, the size of the object and the decompressed content of the object.
#[derive(Debug)]
pub struct PackfileEntry {
    pub obj_type: ObjectType,
    pub size: usize,
    pub content: Vec<u8>,
}

impl PackfileEntry {
    /// Create a new packfile entry.
    pub fn new(obj_type: ObjectType, size: usize, content: Vec<u8>) -> Self {
        Self {
            obj_type,
            size,
            content,
        }
    }

    /// Create a new packfile entry from a hash.
    /// The hash is used to find the file in the .git/objects directory.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the object.
    /// * `git_dir` - The path to the .git directory.
    pub fn from_hash(hash: &str, git_dir: &str) -> io::Result<Self> {
        let file_dir = format!("{}/objects/{}", git_dir, &hash[..2]);
        let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
        let mut decompressor = ZlibDecoder::new(BufReader::new(file));
        let mut decompressed_content = Vec::new();
        decompressor.read_to_end(&mut decompressed_content)?;

        let mut reader = BufReader::new(decompressed_content.as_slice());
        let mut type_buf = Vec::new();
        reader.read_until(b' ', &mut type_buf)?;
        let obj_type = from_utf8(&type_buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        let obj_type = ObjectType::try_from(obj_type.trim())?;

        let mut size_buf = Vec::new();
        reader.read_until(0, &mut size_buf)?;
        let size = from_utf8(&size_buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        let size = size
            .trim_end_matches('\0')
            .parse::<usize>()
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        let mut decompressed_content = Vec::new();
        reader.read_to_end(&mut decompressed_content)?;

        Ok(Self::new(obj_type, size, decompressed_content))
    }
}
