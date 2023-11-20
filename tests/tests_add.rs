use messi::{add, index, rm};
use std::fs;
use std::io::{self, Write};

const GIT_DIR: &str = ".mgit";

fn write_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    write!(file, "{}", content)
}

#[test]
fn test_add_file_is_in_index() -> io::Result<()> {
    fs::create_dir_all(".mgit")?;

    let index_path = ".mgit/index1";
    write_file(index_path, "")?;
    // Given a file with a path
    let path = "tests/add/dir_to_add/non_empty/a.txt";
    write_file(path, "")?;

    // When added to staging area
    add::add(path, index_path, GIT_DIR, "", None)?;

    // Then it is saved in index file
    let index = index::Index::load(index_path, GIT_DIR, "")?;
    assert!(index.contains(path));

    Ok(())
}

#[test]
fn test_update_file() -> io::Result<()> {
    fs::create_dir_all(".mgit")?;

    let index_path = ".mgit/index2";
    write_file(index_path, "")?;
    // Given a file with a path
    let path = "tests/add/dir_to_add/non_empty/b.txt";
    write_file(path, "a new file!")?;

    // When added to staging area
    add::add(path, index_path, GIT_DIR, "", None)?;
    // And after that it is modified
    write_file(path, "an updated file!")?;
    let index = index::Index::load(index_path, GIT_DIR, "")?;
    let first_hash = index.get_hash(path).unwrap();
    // And added again
    add::add(path, index_path, GIT_DIR, "", None)?;
    let index = index::Index::load(index_path, GIT_DIR, "")?;
    let updated_hash = index.get_hash(path).unwrap();

    // Then its hash is updated
    assert_ne!(first_hash, updated_hash);

    Ok(())
}

#[test]
fn test_removing_file() -> io::Result<()> {
    fs::create_dir_all(".mgit")?;

    let index_path = ".mgit/index3";
    write_file(index_path, "")?;
    // Given a file with a path
    let path = "tests/add/dir_to_add/non_empty/d.txt";
    write_file(path, "a new file!")?;
    // When added to staging area
    add::add(path, index_path, GIT_DIR, "", None)?;
    // Then the file is in the index
    let index_before_removal = index::Index::load(index_path, GIT_DIR, "")?;
    assert!(index_before_removal.contains(path));

    // And after it is deleted
    rm::git_rm(path, index_path, GIT_DIR, "")?;
    // Then the file is no longer in the index
    let index_after_removal = index::Index::load(index_path, GIT_DIR, "")?;
    assert!(!index_after_removal.contains(path));

    Ok(())
}
