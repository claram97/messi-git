use api_rust::handlers;
use api_rust::utils::method::Method;
use api_rust::utils::mime_type::MimeType;
use api_rust::utils::request::Request;
use api_rust::utils::response::Response;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

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
    let request = Request::new(&request);
    let request_path_splitted = request.path.split('/').collect::<Vec<&str>>();

    let (status_code, body) = match request.method {
        Method::GET => handlers::get::handle(&request_path_splitted[1..]),
        Method::POST => handlers::post::handle(&request_path_splitted[1..]),
        Method::PUT => handlers::put::handle(&request_path_splitted[1..]),
        Method::PATCH => handlers::patch::handle(&request_path_splitted[1..]),
    };

    let mime_type = get_mime_type(request.headers.get("Accept"));
    let response = Response::new(status_code, body, mime_type);
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

fn run() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    println!("Servidor escuchando en 127.0.0.1:3000...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    let _ = handle_client(stream);
                    println!("Terminando conexión...");
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}
