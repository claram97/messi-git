use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
    thread,
};
/// Logger is a struct that allows to write a logging file while
/// working with multiple threads
///
/// Logger is thought to be used from multiple threads that will log
/// messages at the same time
///
/// WARN: we do not recommend to use whether "write!" or "writeln!" macros.
/// The reason is the internal behavior of them, which have internal steps and do not
/// perform write once, but many times. Logger will lock the resource while writting and
/// will release once it is done. If many calls to write are happening at the same time,
/// the result is unexpected.
///
/// ALTERNATIVE: you could first format the desired string and then call "write!". In this way
/// only one call to write will be performed.
///
/// ```
/// use std::fs;
/// use std::io::Write;
///
/// fn test() -> std::io::Result<()> {
///     let path = "tests/logs/log.txt";
///     let content = "testing log";
///
///     let mut logger = messi::logger::Logger::new(path)?;
///     logger.write(content.as_bytes())?;
///
///     let file_content = fs::read_to_string(path)?;
///     assert_eq!(file_content, content);
///
///     logger.clear()
/// }
/// test();
/// ```
pub struct Logger {
    file: Arc<Mutex<File>>,
}

impl Logger {
    /// Given a path to a file, will create a Logger entity.
    ///
    /// Will return an error if fails while trying to open/create the file.
    ///
    /// Will return a Ok(Logger) in case of success.
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    /// This method clears the content of the logger file
    /// setting its length to zero.
    pub fn clear(&mut self) -> std::io::Result<()> {
        let file_clone = self.file.clone();

        let _ = thread::spawn(move || -> std::io::Result<()> {
            if let Ok(file) = file_clone.lock() {
                file.set_len(0)?
            }
            Ok(())
        })
        .join();
        Ok(())
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let file_clone = self.file.clone();

        let buf_owned = buf.to_owned();

        let _ = thread::spawn(move || -> std::io::Result<()> {
            if let Ok(mut file) = file_clone.lock() {
                file.write_all(&buf_owned)?;
                file.flush()?;
            }
            Ok(())
        })
        .join();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let file_clone = self.file.clone();

        let _ = thread::spawn(move || -> std::io::Result<()> {
            if let Ok(mut file) = file_clone.lock() {
                file.flush()?;
            }
            Ok(())
        })
        .join();

        Ok(())
    }
}
