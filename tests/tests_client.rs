use messi::cat_file::cat_file_return_content;
use messi::client::Client;
use messi::packfile_handler::Packfile;
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
    let mut client = Client::new(&address, "repo3", "localhost");
    client.upload_pack(vec!["nueva_main".to_string(), "main".to_string(), "bran2".to_string()], ".mgit3", "origin")?;
    Ok(())
}

#[test]
#[ignore]
fn test_receive_pack() -> io::Result<()> {
    let address = "localhost:".to_owned() + PORT;
    let mut client = Client::new(&address, "repo2", "localhost");
    client.receive_pack("bran2", ".mgit3")?;
    Ok(())
}

#[test]
#[ignore]
fn test_packfile() -> io::Result<()> {
    let pack: Vec<u8> = vec![
        1, 80, 65, 67, 75, 0, 0, 0, 2, 0, 0, 0, 4, 59, 120, 156, 75, 202, 201, 79, 82, 48, 97, 200,
        200, 84, 228, 2, 0, 20, 175, 2, 240, 185, 1, 120, 156, 75, 202, 201, 79, 82, 48, 52, 103,
        72, 44, 74, 206, 200, 44, 203, 87, 200, 43, 205, 77, 45, 202, 87, 48, 226, 2, 0, 108, 76,
        8, 38, 161, 5, 120, 156, 1, 81, 0, 174, 255, 116, 114, 101, 101, 32, 55, 51, 0, 49, 48, 48,
        54, 52, 52, 32, 82, 69, 65, 68, 77, 69, 46, 109, 100, 0, 50, 170, 216, 195, 83, 135, 33,
        55, 114, 55, 12, 230, 120, 228, 3, 219, 30, 210, 130, 67, 49, 48, 48, 54, 52, 52, 32, 102,
        105, 108, 101, 46, 116, 120, 116, 0, 176, 41, 85, 173, 204, 137, 170, 37, 132, 181, 182, 1,
        221, 144, 209, 32, 165, 12, 81, 38, 68, 107, 29, 92, 145, 15, 120, 156, 157, 142, 65, 14,
        194, 32, 16, 0, 61, 243, 138, 253, 128, 205, 82, 88, 10, 137, 49, 222, 188, 251, 131, 133,
        46, 149, 196, 138, 161, 244, 255, 26, 253, 129, 183, 185, 204, 100, 82, 93, 215, 210, 97,
        52, 120, 232, 77, 4, 76, 242, 1, 145, 130, 21, 27, 145, 9, 73, 27, 159, 115, 208, 142, 35,
        123, 98, 210, 46, 162, 161, 172, 213, 139, 155, 60, 59, 136, 118, 115, 146, 24, 44, 25,
        237, 113, 102, 198, 73, 166, 140, 6, 173, 78, 227, 136, 52, 5, 59, 37, 63, 147, 226, 189,
        223, 107, 131, 27, 175, 165, 85, 184, 202, 214, 235, 86, 225, 212, 150, 31, 93, 114, 25,
        246, 200, 3, 183, 51, 104, 23, 188, 71, 231, 198, 0, 199, 79, 9, 85, 250, 78, 118, 249, 83,
        87, 188, 52, 89, 42, 228, 242, 16, 245, 6, 150, 71, 72, 39, 99, 97, 50, 51, 57, 97, 52, 48,
        99, 99, 57, 52, 51, 101, 56, 53, 48, 49, 56, 48, 99, 98, 50, 49, 57, 98, 51, 51, 57, 57,
        98, 98, 52, 99, 52, 98, 97, 53, 55, 55,
    ];

    let packfile = Packfile::reader(pack.as_slice())?;

    for p in packfile {
        dbg!(String::from_utf8_lossy(p?.content.as_slice()));
    }
    Ok(())
}
