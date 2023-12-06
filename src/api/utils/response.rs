use std::fmt;

use crate::api::utils::mime_type::MimeType;
use crate::api::utils::status_code::StatusCode;
use crate::api::utils::headers::Headers;


pub struct Response {
    status_code: StatusCode,
    headers: Headers,
    body: Option<String>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<String>, mime_type: MimeType) -> Self {

        let mut headers = Headers::default();
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
        write!(
            f,
            "HTTP/1.1 {}{}{}",
            self.status_code,
            self.headers,
            body
        )
    }
}