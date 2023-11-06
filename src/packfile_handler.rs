use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, Error, Read, Write},
    str::from_utf8,
    vec,
};

use flate2::{bufread::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::Digest;
use sha1::Sha1;

use crate::hash_object;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl ObjectType {
    fn as_byte(&self) -> u8 {
        match self {
            ObjectType::Commit => 1,
            ObjectType::Tree => 2,
            ObjectType::Blob => 3,
            ObjectType::Tag => 4,
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Commit => write!(f, "commit"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tag => write!(f, "tag"),
        }
    }
}

impl TryFrom<&str> for ObjectType {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "commit" => Ok(Self::Commit),
            "tree" => Ok(Self::Tree),
            "blob" => Ok(Self::Blob),
            "tag" => Ok(Self::Tag),
            t => Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsopported object type: {}", t),
            )),
        }
    }
}

impl TryFrom<u8> for ObjectType {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Commit),
            2 => Ok(Self::Tree),
            3 => Ok(Self::Blob),
            4 => Ok(Self::Tag),
            t => Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsopported object type: {}", t),
            )),
        }
    }
}

#[derive(Debug)]
pub struct PackfileEntry {
    pub obj_type: ObjectType,
    pub size: usize,
    pub content: Vec<u8>,
}

impl PackfileEntry {
    pub fn new(obj_type: ObjectType, size: usize, content: Vec<u8>) -> Self {
        Self {
            obj_type,
            size,
            content,
        }
    }
}

#[derive(Debug)]
pub struct Packfile<R>
where
    R: Read,
{
    bufreader: BufReader<R>,
    position: u32,
    total: u32,
}

impl<R> Packfile<R>
where
    R: Read,
{
    pub fn reader(packfile: R) -> io::Result<Self> {
        let mut packfile = Self {
            bufreader: BufReader::new(packfile),
            position: 0,
            total: 0,
        };
        packfile.validate()?;
        packfile.count_objects()?;
        Ok(packfile)
    }

    fn validate(&mut self) -> io::Result<()> {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        self.bufreader.read_exact(&mut [0])?;
        self.bufreader.read_exact(&mut buf)?;

        let signature = from_utf8(&buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        if signature != "PACK" {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packfile signature: {}", signature),
            ));
        }

        self.bufreader.read_exact(&mut buf)?;
        let version = u32::from_be_bytes(buf);

        if version != 2 {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Packfile version not supported: {}. Please use v2.",
                    version
                ),
            ));
        }

        Ok(())
    }

    fn count_objects(&mut self) -> io::Result<()> {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        self.bufreader.read_exact(&mut buf)?;
        self.total = u32::from_be_bytes(buf);
        Ok(())
    }

    fn get_next(&mut self) -> io::Result<PackfileEntry> {
        let mut byte = self.read_byte()?;
        // let obj_type = get_object_type(byte)?;
        let obj_type = ObjectType::try_from((byte & 0x70) >> 4)?;
        let mut obj_size = (byte & 0x0f) as usize;
        let mut bshift: usize = 4;
        while (byte & 0x80) != 0 {
            byte = self.read_byte()?;
            obj_size |= ((byte & 0x7f) as usize) << bshift;
            bshift += 7;
        }

        let mut decompressor = ZlibDecoder::new(&mut self.bufreader);
        let mut obj = vec![];
        let bytes_read = decompressor.read_to_end(&mut obj)?;
        if obj_size != bytes_read {
            println!("type {:?}. bytes:\n{:?}", obj_type, obj);
            return Err(Error::new(
                io::ErrorKind::InvalidInput,
                "Corrupted packfile. Size is not correct",
            ));
        }

        Ok(PackfileEntry::new(obj_type, obj_size, obj))
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf: [u8; 1] = [0];
        self.bufreader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl<R> Iterator for Packfile<R>
where
    R: Read,
{
    type Item = io::Result<PackfileEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.total {
            return None;
        }
        self.position += 1;
        Some(self.get_next())
    }
}

/// Create a Git packfile containing objects specified in a `HashSet`.
///
/// This function generates a Git packfile that contains the objects listed in the provided `HashSet`.
/// A packfile is a binary format used by Git to store multiple Git objects efficiently in a single file.
///
/// # Arguments
///
/// * `objects`: A `HashSet` containing tuples of `(ObjectType, String)`, representing the object type
///   and its identifier (typically a hash).
/// * `git_dir`: A string representing the path to the Git repository directory where objects are stored.
///
/// # Returns
///
/// Returns an `io::Result` containing the generated Git packfile as a `Vec<u8>`. If successful,
/// `Ok(packfile)` is returned, where `packfile` is the binary representation of the packfile;
/// otherwise, an error is returned.
///
pub fn create_packfile_from_set(
    objects: HashSet<(ObjectType, String)>,
    git_dir: &str,
) -> io::Result<Vec<u8>> {
    let mut packfile = vec![];
    packfile.extend(b"PACK");
    let version: [u8; 4] = [0, 0, 0, 2];
    packfile.extend(version);
    let obj_count: [u8; 4] = (objects.len() as u32).to_be_bytes();
    packfile.extend(obj_count);
    append_objects(&mut packfile, objects, git_dir)?;
    Ok(packfile)
}

fn append_objects(
    packfile: &mut Vec<u8>,
    objects: HashSet<(ObjectType, String)>,
    git_dir: &str,
) -> io::Result<()> {
    for object in objects {
        append_object(packfile, object, git_dir)?
    }
    let mut hasher = Sha1::new();
    hasher.update(&packfile);
    let checksum = hasher.finalize();
    packfile.extend(checksum);
    Ok(())
}

fn append_object(
    packfile: &mut Vec<u8>,
    object: (ObjectType, String),
    git_dir: &str,
) -> io::Result<()> {
    let (obj_type, hash) = object;

    let content = decompress_object_into_bytes(&hash, git_dir)?;
    let obj_size = content.len();
    let mut compressor = ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
    compressor.write_all(&content)?;
    let compressed_content = compressor.finish()?;

    let mut encoded_header: Vec<u8> = Vec::new();
    // Combina el tipo de objeto y los 4 bits más bajos del tamaño
    let mut c = (obj_type.as_byte() << 4) | ((obj_size & 0x0F) as u8);
    // Codifica el tamaño restante usando codificación de bytes variable
    let mut size = obj_size >> 4;
    while size > 0 {
        encoded_header.push(c | 0x80);

        c = size as u8 & 0x7F;
        size >>= 7;
    }
    encoded_header.push(c);

    packfile.extend(encoded_header);
    packfile.extend(compressed_content);
    Ok(())
}

fn decompress_object_into_bytes(hash: &str, git_dir: &str) -> io::Result<Vec<u8>> {
    let file_dir = format!("{}/objects/{}", git_dir, &hash[..2]);
    let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = Vec::new();
    decompressor.read_to_end(&mut decompressed_content)?;

    let mut reader = BufReader::new(decompressed_content.as_slice());
    reader.read_until(0, &mut Vec::new())?;
    let mut decompressed_content = Vec::new();
    reader.read_to_end(&mut decompressed_content)?;
    Ok(decompressed_content)
}

pub fn unpack_packfile(packfile: &[u8], git_dir: &str) -> io::Result<()> {
    dbg!(git_dir);
    let packfile = Packfile::reader(packfile)?;
    for entry in packfile {
        let entry = entry?;
        hash_object::store_bytes_array_to_file(
            entry.content,
            git_dir,
            &entry.obj_type.to_string(),
        )?;
    }
    Ok(())
}