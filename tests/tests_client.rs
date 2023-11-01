use std::io;

use messi::client::Client;
const PORT: &str = "9418";

#[test]
#[ignore]
fn test_get_refs() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost")?;
    assert!(!client.get_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_get_refs2() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost")?;
    client.get_refs()?;
    client.get_refs()?;
    assert!(!client.get_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_refs_has_head() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost")?;
    let refs = client.get_refs()?;
    assert!(refs.contains(&"HEAD".to_string()));
    Ok(())
}

#[test]
#[ignore]
fn test_upload_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo2", "localhost")?;
    client.upload_pack(Some("HEAD"), ".mgit2")?;
    Ok(())
}

#[test]
#[ignore]
fn test_receive_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost")?;
    client.receive_pack("refs/heads/adios", ".mgit")?;
    Ok(())
}