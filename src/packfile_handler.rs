use std::{
    io::{self, BufReader, Error, Read},
    str::from_utf8,
    vec
};

use flate2::bufread::ZlibDecoder;

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
        dbg!(self.total);
        Ok(())
    }

    fn get_next(&mut self) -> io::Result<(String, String)> {
        let mut byte = self.read_byte()?;
        let obj_type = get_object_type(byte)?;

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
            println!("type {}. bytes:\n{:?}", obj_type, obj);
            return Err(Error::new(io::ErrorKind::InvalidInput, "Corrupted packfile. Size is not correct"))
        }
        
        let content = String::from_utf8_lossy(&obj);
        Ok((obj_type, content.to_string()))
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf: [u8; 1] = [0];
        self.bufreader.read_exact(&mut buf)?;
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
    type Item = io::Result<(String, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.total {
            return None;
        }
        self.position += 1;
        Some(self.get_next())
    }
}
