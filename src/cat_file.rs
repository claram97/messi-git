use std::{io::{self, BufReader, Read}, fs::File};

use flate2::bufread::ZlibDecoder;

pub fn cat_file(hash: &str) -> io::Result<String> {
    let file = File::open(format!("objects/{}", hash))?;
    let content = decompress_file(file)?;
    println!("{}", content);
    Ok(content)
}

fn decompress_file(file: File) -> io::Result<String> {
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = String::new();
    decompressor.read_to_string(&mut decompressed_content)?;
    Ok(decompressed_content)
}
