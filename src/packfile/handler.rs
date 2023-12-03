use std::{
    io::{self, BufReader, Cursor, Error, Read, Seek, Write},
    str::from_utf8,
    vec,
};

use flate2::{bufread::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::Digest;
use sha1::Sha1;

use crate::{hash_object, server_utils::*};

use super::{delta_utils, entry::PackfileEntry, object_type::ObjectType};

/// A packfile reader.
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
        log("Creating packfile reader...")?;
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
        log("Validating packfile...")?;
        let [_] = self.read_bytes()?;
        let buf: [u8; 4] = self.read_bytes()?;

        let signature = from_utf8(&buf)
            .map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        if signature != "PACK" {
            log(&format!("Invalid packfile signature: {}", signature))?;
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packfile signature: {}", signature),
            ));
        }

        let buf = self.read_bytes()?;
        let version = u32::from_be_bytes(buf);

        if version != 2 {
            log(&format!(
                "Packfile version not supported: {}. Please use v2.",
                version
            ))?;
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
        log(&format!("Total objects: {}", self.total))?;
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
    fn get_next(&mut self) -> io::Result<PackfileEntry> {
        let initial_position = self.bufreader.stream_position()?;
        let (obj_type, obj_size) = self.get_obj_type_size()?;
        match obj_type {
            ObjectType::OfsDelta => self.get_ofs_delta_object(initial_position),
            ObjectType::RefDelta => self.get_ref_delta_object(),
            _ => self.get_base_object(obj_type, obj_size),
        }
    }

    /// Reads a non delta object from the packfile.
    ///
    /// This method reads the compressed object data, decompresses it, and constructs a `PackfileEntry`
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if there is an issue reading or decompressing the object.
    fn get_base_object(
        &mut self,
        obj_type: ObjectType,
        obj_size: usize,
    ) -> io::Result<PackfileEntry> {
        let mut decompressor = ZlibDecoder::new(&mut self.bufreader);
        let mut obj = vec![];
        let bytes_read = decompressor.read_to_end(&mut obj)?;
        if obj_size != bytes_read {
            return Err(Error::new(
                io::ErrorKind::InvalidInput,
                "Corrupted packfile. Size is not correct",
            ));
        }
        Ok(PackfileEntry::new(obj_type, obj_size, obj))
    }

    /// Reads an ofs delta object from the packfile.
    ///
    /// # Arguments
    ///
    /// * `initial_position` - The position of the ofs delta object in the packfile.
    fn get_ofs_delta_object(&mut self, initial_position: u64) -> io::Result<PackfileEntry> {
        let delta_offset = delta_utils::read_offset_encoding(&mut self.bufreader)?;
        let base_obj_pos = initial_position
            .checked_sub(delta_offset)
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid delta offset",
            ))?;
        let position = self.bufreader.stream_position()?;
        self.bufreader.seek(io::SeekFrom::Start(base_obj_pos))?;
        let base_object = self.get_next()?;
        self.bufreader.seek(io::SeekFrom::Start(position))?;
        self.apply_delta(&base_object)
    }

    /// Reads a ref delta object from the packfile.
    ///
    /// This method reads the hash of the base object, finds the base object in the packfile, and
    /// applies the delta to the base object.
    ///
    /// THIS METHOD COULD NOT BE TESTED BECAUSE WE COULD NOT FIND A REF DELTA OBJECT IN ANY PACKFILE
    fn get_ref_delta_object(&mut self) -> io::Result<PackfileEntry> {
        let mut hash = [0; 20];
        self.bufreader.read_exact(&mut hash)?;
        let hash: Vec<String> = hash.iter().map(|byte| format!("{:02x}", byte)).collect();
        let hash = hash.concat().to_string();
        let base_object = PackfileEntry::from_hash(&hash, &self.git_dir)?;
        self.apply_delta(&base_object)
    }

    /// Reads the object type and size from the packfile.
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
        log(&format!("Object type: {}", obj_type))?;
        Ok((obj_type, obj_size))
    }

    /// Reads a single byte from the packfile.
    fn read_byte(&mut self) -> io::Result<u8> {
        let [buf] = self.read_bytes()?;
        Ok(buf)
    }

    /// Reads a fixed number of bytes from the packfile.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if there is an issue reading the bytes.
    fn read_bytes<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut bytes = [0; N];
        self.bufreader.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Given a base object and a delta object, applies the delta commands to the base object and returns the
    /// resulting object.
    ///
    /// # Arguments
    ///
    /// * `base` - The base object.
    fn apply_delta(&mut self, base: &PackfileEntry) -> io::Result<PackfileEntry> {
        log("Applying delta...")?;
        let mut delta = ZlibDecoder::new(&mut self.bufreader);
        let base_size = delta_utils::read_size_encoding(&mut delta)?;
        if base.size != base_size {
            let error_message = format!(
                "Incorrect base object length. Expected: {}, Actual: {}",
                base_size, base.size
            );
            log(&error_message)?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
        }
        let result_size = delta_utils::read_size_encoding(&mut delta)?;
        let commands = delta_utils::read_delta_commands(&mut delta)?;
        let result = delta_utils::recreate_from_commands(&base.content, &commands);
        if result.len() != result_size {
            let error_message = format!(
                "Incorrect result object length. Expected: {}, Actual: {}",
                result_size,
                result.len()
            );
            log(&error_message)?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
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
        log(&format!(
            "Reading object {} of {}. Packfile position: {}",
            self.position,
            self.total,
            self.bufreader.stream_position().unwrap_or_default()
        ))
        .ok();
        Some(self.get_next())
    }
}

/// Creates a Git packfile from an array of objects hashes.
///
/// # Arguments
///
/// * `objects` - An array of objects hashes.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` containing the packfile if successful, or an `io::Error` if there is an issue
pub fn create_packfile(objects: &[String], git_dir: &str) -> io::Result<Vec<u8>> {
    log("Creating packfile...")?;
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
/// # Arguments
///
/// * `packfile` - The packfile to unpack.
/// * `git_dir` - The path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading or storing the objects.
pub fn unpack_packfile(packfile: &[u8], git_dir: &str) -> io::Result<()> {
    let packfile = Packfile::reader(Cursor::new(packfile), git_dir)?;
    for entry in packfile {
        let entry = entry?;
        let hash = hash_object::store_bytes_array_to_file(
            entry.content,
            git_dir,
            &entry.obj_type.to_string(),
        )?;
        log(&format!(
            "Object {} of type {} unpacked",
            hash, entry.obj_type
        ))?;
    }
    Ok(())
}

/// Appends objects to the given `packfile` vector.
///
/// # Arguments
///
/// * `packfile` - The packfile vector.
/// * `objects` - An array of objects hashes.
/// * `git_dir` - The path to the Git directory. Used for finding base objects in ref-deltas.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading or appending objects.
fn append_objects(packfile: &mut Vec<u8>, objects: &[String], git_dir: &str) -> io::Result<()> {
    log(&format!("Appending {} objects...", objects.len()))?;
    let mut objects_in_packfile = Vec::new();
    for hash in objects {
        let entry = PackfileEntry::from_hash(hash, git_dir)?;
        let offset = packfile.len();

        let obj_type = match find_base_object_index(&entry, &objects_in_packfile) {
            Some(base_obj) => {
                append_delta_object(packfile, base_obj, &entry, git_dir)?;
                ObjectType::OfsDelta
            }
            None => {
                append_object(packfile, &entry, git_dir)?;
                entry.obj_type
            }
        };
        log(&format!(
            "Object {} of type {} appended at offset {}",
            hash, obj_type, offset
        ))?;
        objects_in_packfile.push((entry, offset));
    }
    let mut hasher = Sha1::new();
    hasher.update(&packfile);
    let checksum = hasher.finalize();
    packfile.extend(checksum);
    Ok(())
}

/// Finds the base object for a given object.
///
/// # Arguments
///
/// * `object` - The object to find the base object for.
/// * `objects` - The objects in the packfile.
///
/// # Returns
///
/// Returns a base object if found a valid candidate, or `None` if not.
fn find_base_object_index<'a>(
    object: &'a PackfileEntry,
    objects: &'a [(PackfileEntry, usize)],
) -> Option<&'a (PackfileEntry, usize)> {
    let toleration = 20;

    if let Some(candidate) = objects
        .iter()
        .min_by_key(|(obj, _)| object.size.abs_diff(obj.size))
    {
        if object.obj_type != candidate.0.obj_type {
            return None;
        }
        if (object.size.abs_diff(candidate.0.size) / object.size) > toleration / 100 {
            return None;
        }
        if enough_candidate_coincidences(object, &candidate.0, toleration) {
            return Some(candidate);
        }
    }
    None
}

/// Checks if two objects have enough coincidences to be considered a valid candidate.
///
/// # Arguments
///
/// * `object` - The object to check.
/// * `candidate` - The candidate object.
/// * `toleration` - The toleration percentage.
///
/// # Returns
///
/// Returns `true` if the objects have enough coincidences, or `false` if not.
fn enough_candidate_coincidences(
    object: &PackfileEntry,
    candidate: &PackfileEntry,
    toleration: usize,
) -> bool {
    let mut total_lines = 0;
    let mut coincidences = 0;
    let mut skip_lines = 0;
    for line in object.content.split(|&c| c == b'\n') {
        let mut lines_read = 0;
        total_lines += 1;
        if candidate
            .content
            .split(|&c| c == b'\n')
            .skip(skip_lines)
            .any(|line2| {
                lines_read += 1;
                line == line2
            })
        {
            skip_lines += lines_read;
            coincidences += 1;
        }
    }

    if coincidences > total_lines * ((100 - toleration) / 100) {
        return true;
    }
    false
}

/// Appends a delta object to the given `packfile` vector.
///
/// # Arguments
///
/// * `packfile` - The packfile vector.
/// * `base_object` - The base object.
/// * `object` - The delta object.
/// * `git_dir` - The path to the Git directory.
fn append_delta_object(
    packfile: &mut Vec<u8>,
    base_object: &(PackfileEntry, usize),
    object: &PackfileEntry,
    _git_dir: &str,
) -> io::Result<()> {
    let offset = packfile.len() - base_object.1;
    let encoded_header = object_header(ObjectType::OfsDelta, 7);
    packfile.extend(encoded_header);

    let encoded_offset = delta_utils::encode_offset(offset);
    packfile.extend(encoded_offset);

    let encoded_base_size = delta_utils::encode_size(base_object.0.size);
    let encoded_result_size = delta_utils::encode_size(object.size);
    let commands =
        delta_utils::delta_commands_from_objects(&base_object.0.content, &object.content);
    let mut encoder = ZlibEncoder::new(packfile, Compression::default());
    encoder.write_all(&encoded_base_size)?;
    encoder.write_all(&encoded_result_size)?;

    for command in commands {
        let encoded = command.encode();
        encoder.write_all(&encoded)?;
    }
    encoder.finish()?;
    log(&format!(
        "Delta object appended. Base object offset: {}",
        base_object.1
    ))?;
    Ok(())
}
/// Appends a single object to the given `packfile` vector.
///
/// # Arguments
///
/// * `packfile` - The packfile vector.
/// * `object` - The object to append.
/// * `git_dir` - The path to the Git directory.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue reading, compressing, or appending the object.
///
fn append_object(packfile: &mut Vec<u8>, object: &PackfileEntry, _git_dir: &str) -> io::Result<()> {
    let encoded_header = object_header(object.obj_type, object.size);
    packfile.extend(encoded_header);

    let mut compressor = ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
    compressor.write_all(&object.content)?;
    let compressed_content = compressor.finish()?;
    packfile.extend(compressed_content);
    log(&format!("Object appended. Size: {}", object.size))?;
    Ok(())
}

/// Creates the header for a packfile object.
///
/// # Arguments
///
/// * `obj_type` - The object type.
/// * `obj_size` - The object size.
///
/// # Returns
///
/// Returns a vector containing the encoded header.
fn object_header(obj_type: ObjectType, obj_size: usize) -> Vec<u8> {
    let mut encoded_header: Vec<u8> = Vec::new();
    let mut c = (obj_type.as_byte() << 4) | ((obj_size & 0x0F) as u8);
    let mut size = obj_size >> 4;
    while size > 0 {
        encoded_header.push(c | 0x80);

        c = size as u8 & 0x7F;
        size >>= 7;
    }
    encoded_header.push(c);
    encoded_header
}
