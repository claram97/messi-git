use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader, Cursor, Error, Read, Seek, Write},
    str::from_utf8,
    vec,
};

use flate2::{bufread::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::Digest;
use sha1::Sha1;

use crate::hash_object;

use super::{object_type::ObjectType, entry::PackfileEntry, delta_utils};

#[derive(Debug)]
pub struct Packfile<R: Read + Seek> {
    bufreader: BufReader<R>,
    position: u32,
    total: u32,
    git_dir: String,
}

impl<R: Read + Seek> Packfile<R> {
    /// Creates a new `PackfileReader` from the provided reader.
    ///
    /// This function initializes a `PackfileReader` with the given reader, validating the packfile format,
    /// counting the number of objects, and setting the initial position.
    ///
    /// # Arguments
    ///
    /// * `packfile` - A type implementing the `Read` trait, representing the packfile data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the initialized `PackfileReader` if successful, or an `io::Error`
    /// if validation or counting fails.
    ///
    pub fn reader(packfile: R, git_dir: &str) -> io::Result<Self> {
        let mut packfile = Self {
            bufreader: BufReader::new(packfile),
            position: 0,
            total: 0,
            git_dir: git_dir.to_string(),
        };
        packfile.validate()?;
        packfile.count_objects()?;
        Ok(packfile)
    }

    /// Validates the format and version of the packfile.
    ///
    /// This method reads the initial bytes from the provided reader, checks the signature and version,
    /// and returns an `io::Result<()>` indicating success or an error if the packfile is invalid.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` with `InvalidData` kind if the packfile signature is not "PACK" or if
    /// the version is not 2.
    ///
    fn validate(&mut self) -> io::Result<()> {
        let [_] = self.read_bytes()?;
        let buf: [u8; 4] = self.read_bytes()?;

        let signature = from_utf8(&buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        if signature != "PACK" {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packfile signature: {}", signature),
            ));
        }

        let buf = self.read_bytes()?;
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

    /// Reads and counts the total number of objects in the packfile.
    ///
    /// This method reads the 4-byte total object count from the provided reader and sets the
    /// `total` field in the `PackfileReader` instance.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if there is an issue reading the total object count.
    ///
    fn count_objects(&mut self) -> io::Result<()> {
        let buf = self.read_bytes()?;
        self.total = u32::from_be_bytes(buf);
        Ok(())
    }

    /// Reads the next object from the packfile and returns a `PackfileEntry`.
    ///
    /// This method reads the object type and size information from the packfile and then reads the
    /// compressed object data. It decompresses the data and constructs a `PackfileEntry` with the
    /// object type, size, and uncompressed data.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if there is an issue reading or decompressing the object.
    ///
    fn get_next(&mut self) -> io::Result<PackfileEntry> {
        let initial_position = self.bufreader.stream_position()?;
        let (obj_type, obj_size) = self.get_obj_type_size()?;

        dbg!(obj_type, obj_size);
        match obj_type {
            ObjectType::OfsDelta => {
                let delta_offset = self.read_offset_encoding()?;
                let position = self.bufreader.stream_position()?;
                self.bufreader
                    .seek(io::SeekFrom::Start(initial_position - delta_offset))?;
                let base_object = self.get_next()?;
                self.bufreader.seek(io::SeekFrom::Start(position))?;
                self.apply_delta(&base_object)
            }
            ObjectType::RefDelta => {
                let mut hash = [0; 20];
                self.bufreader.read_exact(&mut hash)?;
                let hash: Vec<String> = hash.iter().map(|byte| format!("{:02x}", byte)).collect(); // convierto los bytes del hash a string
                let hash = hash.concat().to_string();
                let base_object = PackfileEntry::from_hash(&hash, &self.git_dir)?;
                self.apply_delta(&base_object)
            }
            _ => {
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
        }
    }

    fn get_obj_type_size(&mut self) -> io::Result<(ObjectType, usize)> {
        let mut byte = self.read_byte()?;
        let obj_type = ObjectType::try_from((byte & 0x70) >> 4)?;
        let mut obj_size = (byte & 0x0f) as usize;
        let mut bshift: usize = 4;
        while (byte & 0x80) != 0 {
            byte = self.read_byte()?;
            obj_size |= ((byte & 0x7f) as usize) << bshift;
            bshift += 7;
        }
        Ok((obj_type, obj_size))
    }

    fn read_varint_byte(&mut self) -> io::Result<(u8, bool)> {
        let byte = self.read_byte()?;
        Ok((byte & 0x7f, byte & 0x80 != 0))
    }

    fn read_offset_encoding(&mut self) -> io::Result<u64> {
        let mut value = 0;
        loop {
            let (byte_value, more_bytes) = self.read_varint_byte()?;
            value = (value << 7) | byte_value as u64;
            if !more_bytes {
                return Ok(value);
            }
            value += 1;
        }
    }

    /// Reads a single byte from the packfile.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if there is an issue reading the byte.
    ///
    fn read_byte(&mut self) -> io::Result<u8> {
        let [buf] = self.read_bytes()?;
        Ok(buf)
    }
    fn read_bytes<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut bytes = [0; N];
        self.bufreader.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn apply_delta(&mut self, base: &PackfileEntry) -> io::Result<PackfileEntry> {
        let mut delta = ZlibDecoder::new(&mut self.bufreader);
        let base_size = delta_utils::read_size_encoding(&mut delta)?;
        if base.size != base_size {
            return Err(delta_utils::make_error("Incorrect base object length"));
        }

        let result_size = delta_utils::read_size_encoding(&mut delta)?;
        let mut result = Vec::with_capacity(result_size);
        while delta_utils::apply_delta_instruction(&mut delta, &base.content, &mut result)? {}
        if result.len() != result_size {
            return Err(delta_utils::make_error("Incorrect object length"));
        }
        Ok(PackfileEntry {
            obj_type: base.obj_type,
            size: result_size,
            content: result,
        })
    }
}

impl<R: Read + Seek> Iterator for Packfile<R> {
    type Item = io::Result<PackfileEntry>;

    /// Advances the packfile reader to the next object entry.
    ///
    /// If there are no more objects to read, returns `None`.
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

/// Unpacks a Git packfile into individual Git objects.
///
/// The `packfile` parameter is a slice containing the content of the Git packfile.
///
/// The `git_dir` parameter is the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading or storing the objects.
///
pub fn unpack_packfile(packfile: &[u8], git_dir: &str) -> io::Result<()> {
    let mut file = File::create("tests/packfiles/pack-ref-delta.pack")?;
    file.write_all(packfile)?;
    dbg!("Pacfile written");
    let packfile = Packfile::reader(Cursor::new(packfile), git_dir)?;
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

/// Appends objects to the given `packfile` vector.
///
/// The `objects` parameter is a set of tuples, each containing an `ObjectType` and the hash of the object.
///
/// The `git_dir` parameter is the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading or appending objects.
///
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

/// Appends a single object to the given `packfile` vector.
///
/// The `object` parameter is a tuple containing an `ObjectType` and the hash of the object.
///
/// The `git_dir` parameter is the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading, compressing, or appending the object.
///
fn append_object(
    packfile: &mut Vec<u8>,
    object: (ObjectType, String),
    git_dir: &str,
) -> io::Result<()> {
    let (obj_type, hash) = object;

    let object = PackfileEntry::from_hash(&hash, git_dir)?;
    let obj_size = object.size;
    let mut compressor = ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
    compressor.write_all(&object.content)?;
    let compressed_content = compressor.finish()?;

    let mut encoded_header: Vec<u8> = Vec::new();
    let mut c = (obj_type.as_byte() << 4) | ((obj_size & 0x0F) as u8);
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
