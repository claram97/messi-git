use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn reason_phrase(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::Created => "CREATED",
            Self::BadRequest => "BAD REQUEST",
            Self::NotFound => "NOT FOUND",
            Self::InternalServerError => "INTERNAL SERVER ERROR",
        }
    }

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

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

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