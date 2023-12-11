use crate::api::utils::headers::Headers;
use crate::api::utils::method::Method;
use crate::api::utils::query_string::QueryString;
use std::io;

/// A struct that holds the data of an HTTP request
///
/// # Fields
///
/// * `method` - A Method enum that holds the method of the request.
/// * `path` - A string slice that holds the path of the request.
/// * `headers` - A Headers struct that holds the headers of the request.
/// * `qs` - A QueryString struct that holds the query strings of the request.
/// * `body` - A string slice that holds the body of the request.
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Headers,
    pub qs: QueryString,
    pub body: String,
}

impl Request {
    /// Create a new Request.
    ///
    /// # Arguments
    ///
    /// * `request` - A string slice that holds the HTTP request to be parsed.
    pub fn new(request: &str) -> Self {
        let mut lines = request.lines();

        let head_line = lines.next().unwrap_or_default();
        let mut parts = head_line.split_whitespace();

        let method = parts.next().unwrap_or_default();
        let method = Method::from(method);

        let path = parts.next().unwrap_or_default();
        let (path, qs) = parse_path(path);

        let mut headers = Headers::new();
        loop {
            let line = lines.next().unwrap_or_default();
            if line.is_empty() {
                break;
            }
            headers.add(line);
        }

        let mut body = String::new();
        loop {
            let line = lines.next().unwrap_or_default();
            body.push_str(line);
            if line.is_empty() {
                break;
            }
        }

        println!("\n{:?}\n", body);

        // Verifica si el cuerpo es XML y realiza la conversión a JSON
        if let Some(content_type) = headers.get("Content-Type") {
            if content_type == "application/xml" {
                if let Ok(json_body) = Self::parse_xml_to_json(&body) {
                    body = json_body;
                }
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

    /// Splits the path of the request into a vector of string slices.
    pub fn get_path_split(&self) -> Vec<&str> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
    }

    fn obtain_tag_content(html: &str, tag: &str) -> Option<String> {
        let tag_start = format!("<{}>", tag);
        let tag_end = format!("</{}>", tag);

        if let Some(start_idx) = html.find(&tag_start) {
            if let Some(end_idx) = html.find(&tag_end) {
                let content = &html[(start_idx + tag_start.len())..end_idx];
                return Some(content.to_string());
            }
        }

        None
    }

    /// Parsea el cuerpo XML a JSON.
    fn parse_xml_to_json(xml_str: &str) -> io::Result<String> {
        let result: Vec<&str> = xml_str.split("   ").collect();
        let elements = result[1..5].to_vec();

        let mut result_string: String = String::new();
        result_string.push_str("{    \"title\": \"");
        if let Some(content) = Self::obtain_tag_content(elements[0], "title") {
            result_string.push_str(&content);
            result_string.push_str("\",    ");
        }
        result_string.push_str("\"description\": \"");
        if let Some(content) = Self::obtain_tag_content(elements[1], "description") {
            result_string.push_str(&content);
            result_string.push_str("\",    ");
        }
        result_string.push_str("\"source_branch\": \"");
        if let Some(content) = Self::obtain_tag_content(elements[2], "source_branch") {
            result_string.push_str(&content);
            result_string.push_str("\",    ");
        }
        result_string.push_str("\"target_branch\": \"");
        if let Some(content) = Self::obtain_tag_content(elements[3], "target_branch") {
            result_string.push_str(&content);
            result_string.push_str("\"}");
        }

        Ok(result_string)
    }
}

/// Parse the path of the request into a string slice and a QueryString struct.
fn parse_path(path: &str) -> (String, QueryString) {
    match path.split_once('?') {
        Some((path, qs)) => (path.to_string(), QueryString::from(qs)),
        None => (path.to_string(), QueryString::new()),
    }
}
