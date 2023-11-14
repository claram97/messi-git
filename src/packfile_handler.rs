use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Error, Read, Seek, Write},
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
    OfsDelta,
    RefDelta,
}

impl ObjectType {
    fn as_byte(&self) -> u8 {
        match self {
            ObjectType::Commit => 1,
            ObjectType::Tree => 2,
            ObjectType::Blob => 3,
            ObjectType::Tag => 4,
            ObjectType::OfsDelta => 6,
            ObjectType::RefDelta => 7,
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
            ObjectType::OfsDelta => write!(f, "ofs-delta"),
            ObjectType::RefDelta => write!(f, "ref-delta"),
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
            6 => Ok(Self::OfsDelta),
            7 => Ok(Self::RefDelta),
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

    pub fn from_hash(hash: &str, git_dir: &str) -> io::Result<Self> {
        let file_dir = format!("{}/objects/{}", git_dir, &hash[..2]);
        let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
        let mut decompressor = ZlibDecoder::new(BufReader::new(file));
        let mut decompressed_content = Vec::new();
        decompressor.read_to_end(&mut decompressed_content)?;

        let mut reader = BufReader::new(decompressed_content.as_slice());

        // get type
        let mut type_buf = Vec::new();
        reader.read_until(b' ', &mut type_buf)?;
        let obj_type = from_utf8(&type_buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        let obj_type = ObjectType::try_from(obj_type.trim())?;

        let mut size_buf = Vec::new();
        reader.read_until(0, &mut size_buf)?;
        let size = from_utf8(&size_buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        let size = usize::from_str_radix(size.trim_end_matches('\0'), 10)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        let mut decompressed_content = Vec::new();
        reader.read_to_end(&mut decompressed_content)?;

        Ok(Self {
            obj_type,
            size,
            content: decompressed_content,
        })
    }
}

#[derive(Debug)]
pub struct Packfile<R: Read + Seek> {
    bufreader: BufReader<R>,
    position: u32,
    total: u32,
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
        let [_] = read_bytes(self.bufreader.get_mut())?;
        let buf: [u8; 4] = read_bytes(self.bufreader.get_mut())?;

        let signature = from_utf8(&buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        if signature != "PACK" {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packfile signature: {}", signature),
            ));
        }

        let buf = read_bytes(self.bufreader.get_mut())?;
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
        let buf = read_bytes(self.bufreader.get_mut())?;
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
                todo!("RefDelta not implemented");
                // let mut hash = [0; 20];
                // self.bufreader.read_exact(&mut hash)?;
                // let base_object = decompress_object_into_bytes(hash, self.git_dir)?;
                // self.apply_delta(&base_object)
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
        let base_size = read_size_encoding(&mut delta)?;
        if base.size != base_size {
            return Err(make_error("Incorrect base object length"));
        }

        let result_size = read_size_encoding(&mut delta)?;
        let mut result = Vec::with_capacity(result_size);
        while apply_delta_instruction(&mut delta, &base.content, &mut result)? {}
        if result.len() != result_size {
            return Err(make_error("Incorrect object length"));
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

const COPY_INSTRUCTION_FLAG: u8 = 1 << 7;
const COPY_OFFSET_BYTES: u8 = 4;
const COPY_SIZE_BYTES: u8 = 3;
const COPY_ZERO_SIZE: usize = 0x10000;

// Read an integer of up to `bytes` bytes.
// `present_bytes` indicates which bytes are provided. The others are 0.
fn read_partial_int<R: Read>(
    stream: &mut R,
    bytes: u8,
    present_bytes: &mut u8,
) -> io::Result<usize> {
    let mut value = 0;
    for byte_index in 0..bytes {
        // Use one bit of `present_bytes` to determine if the byte exists
        if *present_bytes & 1 != 0 {
            let [byte] = read_bytes(stream)?;
            value |= (byte as usize) << (byte_index * 8);
        }
        *present_bytes >>= 1;
    }
    Ok(value)
}

// Reads a single delta instruction from a stream
// and appends the relevant bytes to `result`.
// Returns whether the delta stream still had instructions.
fn apply_delta_instruction<R: Read>(
    stream: &mut R,
    base: &[u8],
    result: &mut Vec<u8>,
) -> io::Result<bool> {
    // Check if the stream has ended, meaning the new object is done
    let instruction = match read_bytes(stream) {
        Ok([instruction]) => instruction,
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
        Err(err) => return Err(err),
    };
    if instruction & COPY_INSTRUCTION_FLAG == 0 {
        // Data instruction; the instruction byte specifies the number of data bytes
        if instruction == 0 {
            // Appending 0 bytes doesn't make sense, so git disallows it
            return Err(make_error("Invalid data instruction"));
        }

        // Append the provided bytes
        let mut data = vec![0; instruction as usize];
        stream.read_exact(&mut data)?;
        result.extend_from_slice(&data);
    } else {
        // Copy instruction
        let mut nonzero_bytes = instruction;
        let offset = read_partial_int(stream, COPY_OFFSET_BYTES, &mut nonzero_bytes)?;
        let mut size = read_partial_int(stream, COPY_SIZE_BYTES, &mut nonzero_bytes)?;
        if size == 0 {
            // Copying 0 bytes doesn't make sense, so git assumes a different size
            size = COPY_ZERO_SIZE;
        }
        // Copy bytes from the base object
        let base_data = base
            .get(offset..(offset + size))
            .ok_or_else(|| make_error("Invalid copy instruction"))?;
        result.extend_from_slice(base_data);
    }
    Ok(true)
}

fn make_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn read_bytes<R: Read, const N: usize>(stream: &mut R) -> io::Result<[u8; N]> {
    let mut bytes = [0; N];
    stream.read_exact(&mut bytes)?;
    Ok(bytes)
}

// Read 7 bits of data and a flag indicating whether there are more
fn read_varint_byte<R: Read>(stream: &mut R) -> io::Result<(u8, bool)> {
    let [byte] = read_bytes(stream)?;
    let value = byte & 0x7f;
    let more_bytes = byte & 0x80 != 0;
    Ok((value, more_bytes))
}

fn read_size_encoding<R: Read>(stream: &mut R) -> io::Result<usize> {
    let mut value = 0;
    let mut length = 0; // the number of bits of data read so far
    loop {
        let (byte_value, more_bytes) = read_varint_byte(stream)?;
        // Add in the data bits
        value |= (byte_value as usize) << length;
        // Stop if this is the last byte
        if !more_bytes {
            return Ok(value);
        }

        length += 7;
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

    let content = decompress_object_into_bytes(&hash, git_dir)?;
    let obj_size = content.len();
    let mut compressor = ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
    compressor.write_all(&content)?;
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

/// Decompresses a Git object into a vector of bytes.
///
/// The `hash` parameter is the hash of the Git object.
///
/// The `git_dir` parameter is the path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue decompressing or reading the object.
///
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
    let packfile = Packfile::reader(Cursor::new(packfile))?;
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
