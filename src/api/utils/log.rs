use std::io::{self, Write};

use crate::{logger, utils::get_current_time};

/// Log a message to the logs/api.log file.
pub fn log(message: &str) -> io::Result<()> {
    let mut logger = logger::Logger::new("logs/api.log")?;
    let message = message
        .replace('\0', "\\0")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    let message = format!("{} - {}", get_current_time(), message);
    write!(logger, "{}", message)?;
    logger.flush()
}
