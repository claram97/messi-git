use std::io;

use crate::api::utils::{log::log, status_code::StatusCode};
use serde_json::json;

/// Handle a GET request.
pub fn handle(path_splitted: &[&str]) -> io::Result<(StatusCode, Option<String>)> {
    match path_splitted {
        ["repos", repo, "pulls"] => {
            let body = list_pull_requests(repo)?;
            Ok((StatusCode::Ok, Some(body)))
        }
        ["repos", repo, "pulls", pull_number] => {
            let body = get_pull_request(repo, pull_number)?;
            Ok((StatusCode::Ok, Some(body)))
        }
        ["repos", repo, "pulls", pull_number, "commits"] => {
            let body = list_pull_request_commits(repo, pull_number)?;
            Ok((StatusCode::Ok, Some(body)))
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn list_pull_requests(repo: &str) -> io::Result<String> {
    log(&format!("Listing pull requests of {}", repo))?;
    let result = json!({
        "message": format!("Listando pull requests de {}", repo)
    })
    .to_string();
    Ok(result)
}

fn get_pull_request(repo: &str, pull_number: &str) -> io::Result<String> {
    log(&format!("Showing pull request {} of {}", pull_number, repo))?;
    let result = json!({
        "message": format!("Mostrando pull request {} de {}", pull_number, repo)
    })
    .to_string();
    Ok(result)
}

fn list_pull_request_commits(repo: &str, pull_number: &str) -> io::Result<String> {
    log(&format!(
        "Listing commits of pull request {} of {}",
        pull_number, repo
    ))?;
    let result = json!({
        "message": format!("Listando commits de pull request {} de {}", pull_number, repo)
    })
    .to_string();
    Ok(result)
}
