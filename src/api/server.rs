use crate::api::handlers;
use crate::api::utils::log::log;
use crate::api::utils::method::Method;
use crate::api::utils::mime_type::MimeType;
use crate::api::utils::request::Request;
use crate::api::utils::response::Response;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn read_request(stream: &mut TcpStream) -> io::Result<String> {
    let mut bufreader = BufReader::new(stream);
    let mut buffer = String::new();
    loop {
        let mut temp_buffer = String::new();
        let read = bufreader.read_line(&mut temp_buffer)?;
        if read < 3 {
            break;
        }
        buffer.push_str(&temp_buffer);
    }
    Ok(buffer)
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let request = read_request(&mut stream)?;
    log(&format!("HTTP Request: {}", request))?;
    let request = Request::new(&request);
    log(&format!("Parsed Request: {:?}", request))?;

    let request_path_splitted = request.get_path_split();
    let (status_code, body) = match request.method {
        Method::GET => handlers::get::handle(&request_path_splitted),
        Method::POST => handlers::post::handle(&request_path_splitted),
        Method::PUT => handlers::put::handle(&request_path_splitted),
        Method::PATCH => handlers::patch::handle(&request_path_splitted),
    };

    let mime_type = get_mime_type(request.headers.get("Accept"));
    let response = Response::new(status_code, body, mime_type);

    if let Some(body) = &response.body {
        log(&format!("Response Body: {}", body))?
    };
    log(&format!("Response: {}", response))?;
    write!(stream, "{}", response)?;
    stream.flush()
}

fn get_mime_type(accept: Option<&str>) -> MimeType {
    match accept {
        Some(accept) => accept
            .split(',')
            .map(|mime| MimeType::try_from(mime.trim()))
            .find_map(Result::ok)
            .unwrap_or(MimeType::JSON),
        None => MimeType::JSON,
    }
}

pub fn run(domain: &str, port: &str, path: &str) -> io::Result<()> {
    std::env::set_current_dir(path)?;

    let address = domain.to_owned() + ":" + port;
    let listener = TcpListener::bind(&address)?;

    log(&format!("Changed working directory to {}", path))?;
    log(&format!("Server listening at {}...", &address))?;

    let mut handles = vec![];
    while let Ok((stream, socket_addr)) = listener.accept() {
        log(&format!("New connection from {}...", socket_addr))?;
        let handle = thread::spawn(move || -> io::Result<()> {
            match handle_client(stream) {
                Ok(_) => log(&format!("End connection from {}...Successful", socket_addr))?,
                Err(e) => log(&format!(
                    "End connection from {}...With error: {}",
                    socket_addr, e
                ))?,
            }
            Ok(())
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}
