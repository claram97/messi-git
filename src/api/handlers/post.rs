use std::io;

use serde_json::json;

use crate::api::utils::{log::log, status_code::StatusCode};

/// Handle a POST request.
pub fn handle(path_splitted: &[&str]) -> io::Result<(StatusCode, Option<String>)> {
    match path_splitted {
        ["repos", repo, "pulls"] => {
            let body = create_pull_request(repo)?;
            Ok((StatusCode::Created, Some(body)))
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn create_pull_request(repo: &str) -> io::Result<String> {
    log(&format!("Creating pull request in {}", repo))?;
    let result = json!({
        "message": format!("Creando pull request en {}", repo)
    })
    .to_string();
    Ok(result)
}
