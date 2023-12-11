use serde_json::json;

use crate::api::handlers;
use crate::api::utils::log::log;
use crate::api::utils::method::Method;
use crate::api::utils::mime_type::MimeType;
use crate::api::utils::request::Request;
use crate::api::utils::response::Response;
use crate::pull_request::Repository;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use super::utils::status_code::StatusCode;

/// Read the HTTP request from the client.
fn read_request(stream: &mut TcpStream) -> io::Result<String> {
    let mut buffer = [0; 1024];
    let mut request = Vec::new();

    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                request.extend_from_slice(&buffer[..bytes_read]);
                if bytes_read < buffer.len() {
                    break;
                }
            }
            _ => break,
        }
    }
    let buffer = String::from_utf8(request).unwrap();
    Ok(buffer)
}

/// Handle a client request.
///
/// Parse the request, handle it and send the response.
fn handle_client(stream: &mut TcpStream, repositories: Arc<Repositories>) -> io::Result<()> {
    let request = read_request(stream)?;
    log(&format!("HTTP Request: {}", request))?;
    let request = Request::new(&request);
    log(&format!("Parsed Request: {:?}", request))?;

    // let request_path_splitted = request.get_path_split();
    let (status_code, body) = match request.method {
        Method::GET => handlers::get::handle(&request, repositories)?,
        Method::POST => handlers::post::handle(&request, repositories)?,
        Method::PUT => handlers::put::handle(&request, repositories)?,
        Method::PATCH => handlers::patch::handle(&request, repositories)?,
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

/// Get the mime type of the response.
/// If the Accept header is not present, the default mime type is JSON.
fn get_mime_type(accept: Option<&str>) -> MimeType {
    match accept {
        Some(accept) => accept
            .split(',')
            .map(|mime| MimeType::try_from(mime.trim()))
            .find_map(Result::ok)
            .unwrap_or(MimeType::JSON),
        None => MimeType::default(),
    }
}

/// Handle an error in the server.
fn handle_error(stream: &mut TcpStream, error: &str) -> io::Result<()> {
    let error_message = json!({
        "error": error.to_string()
    })
    .to_string();

    let response = Response::new(
        StatusCode::InternalServerError,
        Some(error_message),
        MimeType::default(),
    );
    write!(stream, "{}", response)?;
    stream.flush()
}

/// Run the REST API server.
///
/// # Arguments
///
/// * `domain` - The domain of the server
/// * `port` - The port of the server.
/// * `path` - The path where the repositories are stored
pub fn run(domain: &str, port: &str, path: &str) -> io::Result<()> {
    std::env::set_current_dir(path)?;
    if !Path::new("prs").exists() {
        std::fs::create_dir("prs")?;
    }
    let address = domain.to_owned() + ":" + port;
    let listener = TcpListener::bind(&address)?;

    log(&format!("Changed working directory to {}", path))?;

    let repositories = Arc::new(Repositories::load()?);
    log("Loaded repositories.")?;

    log(&format!("Server listening at {}...", &address))?;
    println!("Server listening at {}...", &address);

    let mut handles = vec![];
    while let Ok((mut stream, socket_addr)) = listener.accept() {
        log(&format!("New connection from {}...", socket_addr))?;
        let repositories = repositories.clone();
        let handle = thread::spawn(move || -> io::Result<()> {
            match handle_client(&mut stream, repositories) {
                Ok(_) => log(&format!("End connection from {}...Successful", socket_addr))?,
                Err(e) => {
                    log(&format!(
                        "End connection from {}...With error: {}",
                        socket_addr, e
                    ))?;
                    handle_error(&mut stream, &e.to_string())?;
                }
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

/// A struct that contains all the repositories.
/// It is used to share the repositories between threads.
/// It is also used to load the repositories from the disk.
///
/// # Fields
///
/// * `repositories` - The repositories.
pub struct Repositories {
    repositories: Mutex<HashMap<String, Arc<Mutex<Repository>>>>,
}

impl Repositories {
    /// Get a repository.
    /// If the repository is not present in the repositories, it is loaded from the disk if exists.
    ///
    /// # Arguments
    ///
    /// * `repo` - The name of the repository.
    ///
    /// # Returns
    ///
    /// A Mutex of the repository if it exists, otherwise None.
    pub fn get(&self, repo: &str) -> Option<Arc<Mutex<Repository>>> {
        match self.repositories.lock() {
            Ok(mut repositories) => match repositories.get(repo) {
                Some(repo) => Some(repo.clone()),
                None if repo_exists(repo) => {
                    let repository = Arc::new(Mutex::new(Repository::new(repo)));
                    repositories.insert(repo.to_string(), repository.clone());
                    Some(repository)
                }
                None => None,
            },
            Err(_) => None,
        }
    }

    /// Load the repositories from the disk.
    fn load() -> io::Result<Self> {
        let mut repos = HashMap::new();
        let curdir = std::env::current_dir()?;
        let root_dir = curdir.to_string_lossy().to_string();
        let prs_dir = Path::new(&root_dir).join("prs");
        for entry in fs::read_dir(prs_dir)? {
            let entry = entry?;
            let path = entry.path();
            let repo_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".json"))
                .map(|name| name.to_string());
            if let Some(repo_name) = repo_name {
                let repo = serde_json::from_str(&fs::read_to_string(path)?)?;
                repos.insert(repo_name.to_string(), Arc::new(Mutex::new(repo)));
            }
        }
        Ok(Self {
            repositories: Mutex::new(repos),
        })
    }
}

/// Check if a repository exists in the server
fn repo_exists(repo: &str) -> bool {
    let curdir = match std::env::current_dir() {
        Ok(curdir) => curdir,
        Err(_) => return false,
    };
    let repo_dir = curdir.join(repo);
    repo_dir.exists() && repo_dir.is_dir()
}

/// Get the root directory of the server.
pub fn get_root_dir() -> io::Result<String> {
    let curdir = std::env::current_dir()?;
    Ok(curdir.to_string_lossy().to_string())
}
