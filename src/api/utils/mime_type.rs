use std::fmt;

/// A struct that holds the mime type of a request.
///
/// # Supported mime types
///
/// - application/json
/// - application/xml (not implemented yet)
#[derive(Debug, Default)]
pub enum MimeType {
    #[default]
    JSON,
    XML,
}

impl TryFrom<&str> for MimeType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once('/') {
            Some(("application", extension)) => match extension {
                "json" => Ok(Self::JSON),
                "xml" => Ok(Self::XML),
                _ => Err(format!("Mime type {} not found", value)),
            },
            _ => Err(format!("Mime type {} not found", value)),
        }
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JSON => write!(f, "application/json"),
            Self::XML => write!(f, "application/xml"),
        }
    }
}
