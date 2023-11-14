use std::{io, fs, str::from_utf8};

use messi::packfile;

#[test]
fn test_ofs_delta() -> io::Result<()> {
    let packfile = fs::File::open("tests/packfiles/pack-ofs-delta.pack")?;
    let git_dir = "tests/packfiles/.mgit";
    let packfile = packfile::handler::Packfile::reader(packfile,git_dir)?;
    for p in packfile {
        p?;
    }
    Ok(())
}

#[test]
fn test_ref_deltas() -> io::Result<()> {
    let packfile = fs::File::open("tests/packfiles/pack-2.pack")?;
    let git_dir = "tests/packfiles/.mgit";
    let packfile = packfile::handler::Packfile::reader(packfile,git_dir)?;
    for p in packfile {
        p?;
    }
    Ok(())
}

#[test]
fn test_load_object() -> io::Result<()> {
    let hash = "d4fcb8b438a753430575dc76ac380af0f9a002a4";
    let git_dir = "tests/packfiles/.mgit";
    let entry = packfile::handler::PackfileEntry::from_hash(hash, git_dir)?;
    assert!(from_utf8(&entry.content).is_ok());
    Ok(())
}