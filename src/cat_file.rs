use std::{io::{self, BufReader, Read}, fs::File};

use flate2::bufread::ZlibDecoder;

pub fn cat_file(hash: &str) -> io::Result<String> {
    let file_dir = format!("objects/{}", &hash[..2]);
    let file = File::open(format!("{}/{}", file_dir, &hash[2..]))?;
    let content = decompress_file(file)?;
    let partes = content.split('\0').nth(1);
    match partes {
        Some(partes) => println!("{}", partes),
        None => println!("{}", content),
    }
    Ok(content)
}

fn decompress_file(file: File) -> io::Result<String> {
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = String::new();
    decompressor.read_to_string(&mut decompressed_content)?;
    Ok(decompressed_content)
}
