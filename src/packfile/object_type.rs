use std::{
    fmt::Display,
    io::{self, Error},
};

/// Possible object types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    OfsDelta,
    RefDelta,
}

impl ObjectType {
    /// Get the byte representation of the object type.
    pub fn as_byte(&self) -> u8 {
        match self {
            ObjectType::Commit => 1,
            ObjectType::Tree => 2,
            ObjectType::Blob => 3,
            ObjectType::Tag => 4,
            ObjectType::OfsDelta => 6,
            ObjectType::RefDelta => 7,
        }
    }
}

impl Display for ObjectType {
    /// Get the string representation of the object type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Commit => write!(f, "commit"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tag => write!(f, "tag"),
            ObjectType::OfsDelta => write!(f, "ofs-delta"),
            ObjectType::RefDelta => write!(f, "ref-delta"),
        }
    }
}

impl TryFrom<&str> for ObjectType {
    type Error = io::Error;
    /// Try to get the object type from a string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "commit" => Ok(Self::Commit),
            "tree" => Ok(Self::Tree),
            "blob" => Ok(Self::Blob),
            "tag" => Ok(Self::Tag),
            t => Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsopported object type: {}", t),
            )),
        }
    }
}

impl TryFrom<u8> for ObjectType {
    type Error = io::Error;

    /// Try to get the object type from a byte.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Commit),
            2 => Ok(Self::Tree),
            3 => Ok(Self::Blob),
            4 => Ok(Self::Tag),
            6 => Ok(Self::OfsDelta),
            7 => Ok(Self::RefDelta),
            t => Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsopported object type: {}", t),
            )),
        }
    }
}
