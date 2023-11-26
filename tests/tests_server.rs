use std::{
    fs::File,
    io::{self, BufReader, Read}, str::from_utf8,
};

use flate2::bufread::ZlibDecoder;
use messi::{cat_file::cat_file_return_content, server};
const PORT: &str = "9418";

#[test]
#[ignore]
fn test_run_server() -> io::Result<()> {
    server::run("localhost", PORT, "/home/rgestoso/daemon/server", ".git")
}

#[test]
#[ignore]
fn test_read_file() -> io::Result<()> {
    let file_dir = format!("{}/objects/{}", "tests/packfiles/.mgit", "1c");
    let file = File::open(format!(
        "{}/{}",
        file_dir, "33ce3a1ff4460b096bd992f5e6ff4ea2f80dc1"
    ))?;
    let mut decompressor = ZlibDecoder::new(BufReader::new(file));
    let mut decompressed_content = Vec::new();
    decompressor.read_to_end(&mut decompressed_content)?;
    dbg!(from_utf8(decompressed_content.as_slice()).unwrap());
    dbg!(decompressed_content.len());
    Ok(())
}
