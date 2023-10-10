use std::{
    fs::{File, OpenOptions},
    io:: Write,
    sync::{Arc, Mutex},
    thread,
};

pub struct Logger {
    file: Arc<Mutex<File>>,
}

impl Logger {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.to_string())?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
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
        }).join();

        Ok(())
    }
}
