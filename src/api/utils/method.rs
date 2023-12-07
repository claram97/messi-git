/// HTTP Method
///
/// # Supported Methods
///
/// - GET
/// - POST
/// - PUT
/// - PATCH
#[derive(Debug, PartialEq, Eq, Default)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Self::GET,
            "POST" => Self::POST,
            "PUT" => Self::PUT,
            "PATCH" => Self::PATCH,
            _ => Self::GET,
        }
    }
}
