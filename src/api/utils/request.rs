use std::io;

use serde_json::Value;

use crate::api::utils::headers::Headers;
use crate::api::utils::method::Method;
use crate::api::utils::query_string::QueryString;

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

    /// Parsea el cuerpo XML a JSON.
    fn parse_xml_to_json(xml_str: &str) -> io::Result<String> {

        /*
        <PullRequestCreate>
                <title>{}</title>
                <description>{}</description>
                <source_branch>{}</source_branch>
                <target_branch>{}</target_branch>
            </PullRequestCreate>
            
         */

        let xml_splitted : Vec<&str> = xml_str.split("\n").collect();
        if xml_splitted.len() != 6 {
            //Return error
        }

        let title = xml_splitted[1];
        let description = xml_splitted[2];
        let source_branch = xml_splitted[3];
        let target_branch = xml_splitted[4];

        
        let json_str = "Hola";      
        Ok(json_str.to_string())
    }

    fn validate_xml(title : &str, description : &str, source_branch : &str, target_branch : &str) {
        if !title.contains("<title>") || !description.contains("<description>"){

        }
    }
}

/// Parse the path of the request into a string slice and a QueryString struct.
fn parse_path(path: &str) -> (String, QueryString) {
    match path.split_once('?') {
        Some((path, qs)) => (path.to_string(), QueryString::from(qs)),
        None => (path.to_string(), QueryString::new()),
    }
}
