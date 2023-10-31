use std::{
    io::{self, BufReader, Error, Read},
    str::from_utf8,
    vec,
};

use flate2::bufread::ZlibDecoder;
use sha1::{Digest, Sha1};

pub fn read_pack_file(packfile: &[u8]) -> io::Result<()> {
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    let mut reader = BufReader::new(packfile);

    reader.read_exact(&mut [0])?;
    reader.read_exact(&mut buf)?;

    let signature =
        from_utf8(&buf).map_err(|err| Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    if signature != "PACK" {
        return Err(Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid packfile signature: {}", signature),
        ));
    }

    reader.read_exact(&mut buf)?;
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

    reader.read_exact(&mut buf)?;
    let objects_quantity = u32::from_be_bytes(buf);
    for _ in 0..objects_quantity {
        process_packfile_object(&mut reader)?;
    }
    Ok(())
}

fn read_byte(reader: &mut impl Read) -> io::Result<u8> {
    let mut buf: [u8; 1] = [0];
    reader.read(&mut buf)?;
    Ok(buf[0])
}

fn get_object_type(byte: u8) -> io::Result<String> {
    match (byte & 0x70) >> 4 {
        1 => Ok(String::from("commit")),
        2 => Ok(String::from("tree")),
        3 => Ok(String::from("blob")),
        4 => Ok(String::from("tag")),
        t => Err(Error::new(io::ErrorKind::InvalidData, format!("Unsopported object type: {}", t))),
    }
}

fn process_packfile_object(bufreader: &mut BufReader<&[u8]>) -> io::Result<()> {
    let mut byte = read_byte(bufreader)?;
    let obj_type_number = byte.clone();
    let obj_type = get_object_type(obj_type_number)?;
    println!("Obj type: {}", obj_type);
    let mut obj_size = byte & 0x0f;
    let mut bshift = 4;
    while (byte & 0x80) != 0 {
        byte = read_byte(bufreader)?;
        obj_size |= (byte & 0x7f) << bshift;
        bshift += 7;
    }

    let mut decompressor = ZlibDecoder::new(bufreader);
    let mut obj = vec![];
    decompressor.read_to_end(&mut obj)?;

    let content = String::from_utf8_lossy(&obj);

    println!("{}", content);
    let file_content = format!("{obj_type} {}\0", content.len());
    let mut hasher = Sha1::new();
    hasher.update(file_content.as_bytes());
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    println!("hash generado: {}", format!("{:x}", result));

    Ok(())
}

fn decompress_packfile_object(bufreader: &mut BufReader<&[u8]>) -> io::Result<()> {
    let mut decompressor = ZlibDecoder::new(bufreader);
    let mut obj = vec![];
    decompressor.read_to_end(&mut obj)?;

    Ok(())
}
