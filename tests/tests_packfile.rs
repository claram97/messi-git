use std::{io, fs};

use messi::packfile_handler;

#[test]
fn test_1() -> io::Result<()> {
    let packfile = fs::File::open("tests/packfiles/pack-1.pack")?;
    let packfile = packfile_handler::Packfile::reader(packfile)?;
    for p in packfile {
        dbg!(p?.obj_type);
    }
    Ok(())
}