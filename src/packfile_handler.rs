use std::{
    io::{self, BufReader, Error, Read},
    str::from_utf8,
    vec, fmt::Display,
};

use flate2::bufread::ZlibDecoder;

pub struct PackfileItem {
    obj_type: Option<String>,
    content: String,
}

impl PackfileItem {
    fn new(obj_type: Option<String>, content: &str) -> Self {
        Self {
            obj_type: obj_type,
            content: content.to_string(),
        }
    }

    pub fn obj_type(&self) -> &Option<String> {
        &self.obj_type
    }

    pub fn content(&self) -> &str {
        &self.content
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
    pub fn new(packfile: R) -> io::Result<Self> {
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

    fn get_next(&mut self) -> io::Result<PackfileItem> {
        let mut byte = self.read_byte()?;
        let obj_type_number = byte.clone();
        let obj_type = match get_object_type(obj_type_number) {
            Ok(t) => t,
            Err(_) => return Ok(PackfileItem::new(None, "")),
        };
        let mut obj_size = (byte & 0x0f) as usize;
        let mut bshift = 4;
        while (byte & 0x80) != 0 {
            byte = self.read_byte()?;
            obj_size |= ((byte & 0x7f) << bshift) as usize;
            bshift += 7;
        }

        let mut decompressor = ZlibDecoder::new(&mut self.bufreader);
        let mut obj = vec![];
        decompressor.read_to_end(&mut obj)?;

        let content = String::from_utf8_lossy(&obj);
        Ok(PackfileItem::new(Some(obj_type), &content))
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf: [u8; 1] = [0];
        self.bufreader.read(&mut buf)?;
        Ok(buf[0])
    }
}

fn get_object_type(byte: u8) -> io::Result<String> {
    match (byte & 0x70) >> 4 {
        1 => Ok(String::from("commit")),
        2 => Ok(String::from("tree")),
        3 => Ok(String::from("blob")),
        4 => Ok(String::from("tag")),
        t => Err(Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsopported object type: {}", t),
        )),
    }
}

impl<R> Iterator for Packfile<R>
where
    R: Read,
{
    type Item = PackfileItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.total - 1 {
            return None;
        }
        self.position += 1;

        match self.get_next() {
            Ok(obj) => Some(obj),
            Err(_) => None,
        }
    }
}

impl Display for PackfileItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "type: {:?}\ncontent:\n|{}|", self.obj_type, &self.content)
    }
}
