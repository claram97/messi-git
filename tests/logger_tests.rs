use messi::logger::Logger;
use std::fs;
use std::io::Write;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[test]
fn test_write_single() -> std::io::Result<()> {
    let path = "tests/logs/test_write_single.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    write!(logger, "{content}")?;
    let file_content = fs::read_to_string(path)?;
    assert!(file_content.starts_with(content));

    logger.clear()
}

#[test]
fn test_write_single_many_times() -> std::io::Result<()> {
    let path = "tests/logs/test_write_single_many_times.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    write!(logger, "{content}")?;
    write!(logger, "{content}")?;
    write!(logger, "{content}")?;

    for line_content in fs::read_to_string(path)?.lines() {
        assert_eq!(line_content, content);
    }

    logger.clear()
}

#[test]
fn test_write_and_clear() -> std::io::Result<()> {
    let path = "tests/logs/test_write_and_clear.txt";
    let content = "testing log";
    let mut logger = Logger::new(path)?;

    write!(logger, "{content}")?;
    logger.clear()?;
    let file_content = fs::read_to_string(path)?;
    assert_eq!(file_content, "");

    Ok(())
}

fn write_in_thread(
    path: &str,
    content: &str,
    times: u64,
) -> JoinHandle<Result<(), std::io::Error>> {
    let path_owned = path.to_owned();
    let content_owned = content.to_owned();
    thread::spawn(move || -> std::io::Result<()> {
        let mut logger = Logger::new(&path_owned)?;
        for i in 0..times {
            let content = format!("{} - {}", content_owned, i);
            write!(logger, "{content}")?;
            thread::sleep(Duration::from_millis(1 * times));
        }
        Ok(())
    })
}

#[test]
fn test_write_many_threads() -> std::io::Result<()> {
    let path = "tests/logs/test_write_many_threads.txt";
    let content_thread_1 = "testing log thread 1";
    let content_thread_2 = "testing log thread 2";

    let mut handles = vec![];
    let handle_1 = write_in_thread(path, content_thread_1, 20);
    let handle_2 = write_in_thread(path, content_thread_2, 30);
    handles.push(handle_1);
    handles.push(handle_2);

    for handle in handles {
        let _ = handle.join();
    }

    assert_eq!(fs::read_to_string(path)?.lines().count(), 50);

    Logger::new(path)?.clear()
}
