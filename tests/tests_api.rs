use std::io;

use messi::api::server;
use messi::api::utils::{
    headers::Headers, method::Method, query_string::QueryString, request::Request,
    status_code::StatusCode,
};

#[test]
#[ignore = "This test is for running the server"]
fn test_run_server() -> io::Result<()> {
    let repos_path = "/home/claram97/Desktop/23C2-messi/tests/test_list_commits";
    server::run("localhost", "3000", repos_path)
}

#[test]
fn test_query_strings() {
    let qs = QueryString::from("id=1&not=2");
    let mut qs_example = QueryString::new();
    qs_example.insert("id", "1");
    qs_example.insert("not", "2");
    assert_eq!(qs, qs_example);

    assert!(qs.get("id").is_some());
    assert!(qs.get("not").is_some());
    assert!(qs.get("nope").is_none());
}

#[test]
fn test_status_code() {
    assert_eq!(StatusCode::Ok.to_string(), "200 Ok");
    assert_eq!(StatusCode::BadRequest.to_string(), "400 Bad Request");
    assert_eq!(StatusCode::NotFound.to_string(), "404 Not Found");
    assert_eq!(
        StatusCode::InternalServerError.to_string(),
        "500 Internal Server Error"
    );
}

#[test]
fn test_headers() {
    let headers = Headers::from(vec![
        "Content-Type: application/json",
        "User-Agent: PostmanRuntime/7.32.3",
        "Accept: */*",
        "Postman-Token: 16e283c7-37dd-4885-878f-cbc54a1f3ebf",
        "Host: localhost:3000",
        "Accept-Encoding: gzip, deflate, br",
        "Connection: keep-alive",
        "Content-Length: 44",
    ]);
    let mut header_example = Headers::new();
    header_example.insert("Content-Type", "application/json");
    header_example.insert("user-agent", "PostmanRuntime/7.32.3");
    header_example.insert("Accept", "*/*");
    header_example.insert("postman-token", "16e283c7-37dd-4885-878f-cbc54a1f3ebf");
    header_example.insert("host", "localhost:3000");
    header_example.insert("accept-encoding", "gzip, deflate, br");
    header_example.insert("connection", "keep-alive");
    header_example.insert("content-length", "44");
    assert_eq!(headers, header_example);

    assert!(headers.get("Content-Type").is_some());
    assert!(headers.get("User-Agent").is_some());
    assert!(headers.get("Accept").is_some());
    assert!(headers.get("Postman-Token").is_some());
    assert!(headers.get("host").is_some());
    assert!(headers.get("Accept-encoding").is_some());
    assert!(headers.get("Connection").is_some());
    assert!(headers.get("Content-Length").is_some());
    assert!(headers.get("nope").is_none());
}

#[test]
fn test_request() {
    let request = "POST /items?id=1&not=2 HTTP/1.1\r\nContent-Type: application/json\r\nUser-Agent: PostmanRuntime/7.32.3\r\nAccept: */*\r\nPostman-Token: 16e283c7-37dd-4885-878f-cbc54a1f3ebf\r\nHost: localhost:3000\r\nAccept-Encoding: gzip, deflate, br\r\nConnection: keep-alive\r\nContent-Length: 44\r\n\r\n{\r\n    \"item\": 1,\r\n    \"ean\": \"123456789\"\r\n}";
    let request = Request::new(request);
    assert_eq!(request.method, Method::POST);
    assert_eq!(request.path, "/items");
    assert_eq!(request.qs, QueryString::from("id=1&not=2"));
    assert_eq!(
        request.headers.get("Content-Type"),
        Some("application/json")
    );
    assert_eq!(
        request.headers.get("User-Agent"),
        Some("postmanruntime/7.32.3")
    );
    assert_eq!(request.headers.get("Accept"), Some("*/*"));
    assert_eq!(
        request.headers.get("Postman-Token"),
        Some("16e283c7-37dd-4885-878f-cbc54a1f3ebf")
    );
    assert_eq!(request.headers.get("Host"), Some("localhost:3000"));
    assert_eq!(
        request.headers.get("Accept-Encoding"),
        Some("gzip, deflate, br")
    );
    assert_eq!(request.headers.get("Connection"), Some("keep-alive"));
    assert!(request.body.contains("\"item\": 1"));
    assert!(request.body.contains("\"ean\": \"123456789\""));
}
