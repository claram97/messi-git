use messi::logger::Logger;
use std::fs;
use std::io::Write;

#[test]
fn test_write_single() -> std::io::Result<()> {
    let path = "tests/logs/log.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    write!(logger, "{content}")?;
    let file_content = fs::read_to_string(path)?;
    assert_eq!(file_content, content);

    logger.clear()
}

#[test]
fn test_write_single_many_times() -> std::io::Result<()> {
    let path = "tests/logs/log.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    writeln!(logger, "{content}")?;
    writeln!(logger, "{content}")?;
    writeln!(logger, "{content}")?;
    
    for line_content in fs::read_to_string(path)?.lines() {
        assert_eq!(line_content, content);
    }

    logger.clear()
}

#[test]
fn test_write_and_clear() -> std::io::Result<()> {
    let path = "tests/logs/log.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    writeln!(logger, "{content}")?;
    logger.clear()?;
    let file_content = fs::read_to_string(path)?;
    assert_eq!(file_content, "");

    Ok(())
}
