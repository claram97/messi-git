use std::{
    collections::HashSet,
    fmt::Display,
    io::{self, BufReader, Error, Read, Write},
    str::from_utf8,
    vec,
};

use flate2::{
    bufread::ZlibDecoder,
    write::ZlibEncoder,
    Compression,
};
use sha1::Sha1;
use sha1::Digest;

use crate::cat_file::cat_file_return_content;

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
        dbg!(self.total);
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

pub fn create_packfile_from_set(
    objects: HashSet<(ObjectType, String)>,
    git_dir: &str,
) -> io::Result<Vec<u8>> {
    let mut packfile = vec![];
    // header start (ver si va aca o no)
    packfile.push(1);
    packfile.push(b'P');
    packfile.push(b'A');
    packfile.push(b'C');
    packfile.push(b'K');
    let version: [u8; 4] = (2 as u32).to_be_bytes();
    packfile.extend(version);
    let obj_count: [u8; 4] = (objects.len() as u32).to_be_bytes();
    packfile.extend(obj_count);
    append_objects(&mut packfile, objects, git_dir)?;
    // ver lo del checksum aca al final
    Ok(packfile)
}

fn append_objects(
    packfile: &mut Vec<u8>,
    objects: HashSet<(ObjectType, String)>,
    git_dir: &str,
) -> io::Result<()> {
    for (obj_type, hash) in objects {
        
        let content = cat_file_return_content(&hash, git_dir)?;
        let size = content.len() as u64;
        let mut compressor = ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
        compressor.write_all(content.as_bytes())?;
        let compressed_content = compressor.finish()?;
        
        
        let t = (obj_type.as_byte() << 4) & 0x70;
        packfile.push(t);
        packfile.extend(encode_varint(size));
        packfile.extend(compressed_content);
        // ver si hace falta meterle un EOF
    }
    let mut hasher = Sha1::new();
    hasher.update(&packfile);
    let result = hasher.finalize();
    write!(packfile, "{:02x}", result) // packfile.extend(result)
    // Ok(())
}

// 
fn encode_varint(mut n: u64) -> Vec<u8> {
    let mut bytes = Vec::new();

    while n >= 0x80 {
        let byte = ((n & 0x7F) | 0x80) as u8;
        bytes.push(byte);
        n >>= 7;
    }

    bytes.push(n as u8);

    bytes
}
