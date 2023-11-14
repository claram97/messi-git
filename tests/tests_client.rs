use messi::cat_file::cat_file_return_content;
use messi::client::Client;
use std::io;
const PORT: &str = "9418";

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
fn test_get_server_refs() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    assert!(!client.get_server_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_get_server_refs2() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    client.get_server_refs()?;
    client.get_server_refs()?;
    assert!(!client.get_server_refs()?.is_empty());
    Ok(())
}

#[test]
#[ignore]
fn test_refs_has_head() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo3", "localhost");
    let refs = client.get_server_refs()?;
    assert!(refs.contains_key(&"HEAD".to_string()));
    Ok(())
}

#[test]
#[ignore]
fn test_upload_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo", "localhost");
    client.upload_pack(
        vec![
            "master".to_string(),
        ],
        "tests/packfiles/.mgit",
        "origin",
    )?;
    Ok(())
}

#[test]
#[ignore]
fn test_receive_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo-push", "localhost");
    client.receive_pack("master", "tests/packfiles/.mgit")?;
    Ok(())
}
