use std::{io, fs, str::from_utf8};

use messi::packfile_handler::{self, ObjectType};

#[test]
fn test_1() -> io::Result<()> {
    let packfile = fs::File::open("tests/packfiles/pack-1.pack")?;
    let packfile = packfile_handler::Packfile::reader(packfile)?;
    for p in packfile {
        let obj = p?;
        if obj.obj_type == ObjectType::Blob || obj.obj_type == ObjectType::Commit {
            let a = from_utf8(&obj.content).unwrap();
            dbg!(a);
        }
    }
    Ok(())
}