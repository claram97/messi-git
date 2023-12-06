use crate::api::utils::headers::Headers;
use crate::api::utils::method::Method;
use crate::api::utils::query_string::QueryString;

#[derive(Debug, Default)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Headers,
    pub qs: QueryString,
    pub body: String,
}

impl Request {
    pub fn new(request: &str) -> Self {
        let mut lines = request.lines();
        let first_line = lines.next().unwrap_or_default();

        let mut parts = first_line.split_whitespace();

        let method = parts.next().unwrap_or_default();
        let method = Method::from(method);

        let path = parts.next().unwrap_or_default();
        let (path, qs) = match path.split_once("?") {
            Some((path, qs)) => (path.to_string(), QueryString::from(qs)),
            None => (path.to_string(), QueryString::default()),
        };

        let mut headers = Vec::new();
        loop {
            let line = lines.next().unwrap_or_default();
            headers.push(line);
            if line.is_empty() {
                break;
            }
        }
        let headers = Headers::from(headers);

        let mut body = String::new();
        loop {
            let line = lines.next().unwrap_or_default();
            body.push_str(line);
            if line.is_empty() {
                break;
            }
        }
        // transformamos el body segun mime type, ahora es siempre json.
        // pero si viene XML hay que pasarlo a json que es lo que entendemos

        Self {
            method,
            path,
            headers,
            qs,
            body,
        }
    }
}
