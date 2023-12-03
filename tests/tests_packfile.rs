use std::{collections::HashSet, fs, io, str::from_utf8};

use messi::{
    packfile::{self, entry::PackfileEntry, handler::create_packfile},
    server_utils,
};

#[test]
fn test_ofs_delta() -> io::Result<()> {
    let packfile = fs::File::open("tests/packfiles/pack-ofs-delta.pack")?;
    let git_dir = "tests/packfiles/.mgit";
    let packfile = packfile::handler::Packfile::reader(packfile, git_dir)?;
    for p in packfile {
        p?;
    }
    Ok(())
}

#[test]
fn test_load_object() -> io::Result<()> {
    let hash = "d4fcb8b438a753430575dc76ac380af0f9a002a4";
    let git_dir = "tests/packfiles/.mgit";
    let entry = PackfileEntry::from_hash(hash, git_dir)?;
    assert!(from_utf8(&entry.content).is_ok());
    Ok(())
}

#[test]
fn test_create_deltas() -> io::Result<()> {
    let haves = HashSet::new();
    let git_dir = "tests/packfiles/.mgit";
    let missing = server_utils::get_missing_objects_from(
        "86135720c1283d83f2744781a915aba3d74da37b",
        &haves,
        git_dir,
    )?;
    create_packfile(&missing, git_dir)?;
    Ok(())
}
