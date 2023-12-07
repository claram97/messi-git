use std::fmt;

use crate::api::utils::headers::Headers;
use crate::api::utils::mime_type::MimeType;
use crate::api::utils::status_code::StatusCode;

/// A struct that holds the data of an HTTP response
///
/// # Fields
///
/// * `status_code` - A StatusCode enum that holds the status code of the response.
/// * `headers` - A Headers struct that holds the headers of the response.
/// * `body` - A string slice that holds the body of the response.
pub struct Response {
    pub status_code: StatusCode,
    pub headers: Headers,
    pub body: Option<String>,
}

impl Response {
    /// Create a new Response.
    ///
    /// # Arguments
    ///
    /// * `status_code` - A StatusCode enum that holds the status code of the response.
    /// * `body` - A string slice that holds the body of the response.
    /// * `mime_type` - A MimeType enum that holds the mime type of the body.
    pub fn new(status_code: StatusCode, body: Option<String>, mime_type: MimeType) -> Self {
        let mut headers = Headers::new();
        if let Some(b) = &body {
            headers.insert("Content-Type", &mime_type.to_string());
            headers.insert("Content-Length", &b.len().to_string());
        }
        // transformamos el body segun mime type, ahora es siempre json
        Self {
            status_code,
            headers,
            body,
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = match &self.body {
            Some(b) => "\r\n\r\n".to_owned() + b,
            None => "\r\n".to_owned(),
        };
        write!(f, "HTTP/1.1 {}{}{}", self.status_code, self.headers, body)
    }
}
