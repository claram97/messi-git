use std::fmt;

/// An enum that holds the status code of a response.
///
/// # Supported status codes
///
/// - 200 Ok
/// - 201 Created
/// - 400 Bad Request
/// - 404 Not Found
/// - 500 Internal Server Error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    Conflict = 409,
    InternalServerError = 500,
}

impl StatusCode {
    /// Get the reason phrase of a status code.
    pub fn reason_phrase(&self) -> &str {
        match self {
            Self::Ok => "Ok",
            Self::Created => "Created",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::Conflict => "Conflict",
            Self::InternalServerError => "Internal Server Error",
        }
    }

    /// Create a status code from a u16.
    ///
    /// # Arguments
    ///
    /// * `code` - A u16 that holds the status code to be created.
    pub fn from_u16(code: u16) -> Result<Self, Error> {
        match code {
            200 => Ok(Self::Ok),
            201 => Ok(Self::Created),
            400 => Ok(Self::BadRequest),
            404 => Ok(Self::NotFound),
            500 => Ok(Self::InternalServerError),
            _ => Err(Error::InvalidRequest),
        }
    }

    /// Get the u16 value of a status code.
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

/// An enum that holds the errors of a status code.
#[derive(Debug)]
pub enum Error {
    InvalidRequest,
    InvalidEncoding,
    InvalidProtocol,
    InvalidMethod,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.to_u16(), self.reason_phrase())
    }
}
