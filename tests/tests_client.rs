use std::io;

use messi::client::Client;
const PORT: &str = "9418";
use messi::cat_file::cat_file_return_content;

#[test]
#[ignore]
fn test_111() -> io::Result<()> {
    let content = cat_file_return_content("21245646a0ba0748560e90865e838ceafd8306f3", ".mgit2")?;
    dbg!("DAS");
    dbg!(content);
    Ok(())
}


#[test]
#[ignore]
fn test_get_refs() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    assert!(!client.get_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_get_refs2() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    client.get_refs()?;
    client.get_refs()?;
    assert!(!client.get_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_refs_has_head() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    let refs = client.get_refs()?;
    assert!(refs.contains(&"HEAD".to_string()));
    Ok(())
}

#[test]
#[ignore]
fn test_upload_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo2", "localhost");
    client.upload_pack("HEAD", ".mgit3", "origin")?;
    Ok(())
}

#[test]
#[ignore]
fn test_receive_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    client.receive_pack("refs/heads/adios", ".mgit")?;
    Ok(())
}