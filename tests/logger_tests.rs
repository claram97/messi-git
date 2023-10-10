use messi::logger::Logger;
use std::fs;
use std::io::Write;

#[test]
fn test_write_single() -> std::io::Result<()> {
    let path = "tests/logs/log.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;
    dbg!("ACA");
    write!(logger, "{content}")?;
    let file_content = fs::read_to_string(path)?;
    assert_eq!(file_content, content);

    Ok(())
}
