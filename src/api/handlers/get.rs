use std::io;

use crate::{
    api::utils::{log::log, request::Request, status_code::StatusCode},
    configuration::GIT_DIR,
    pull_request::Repository,
};
use serde_json::json;

/// Handle a GET request.
pub fn handle(request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls"] => list_pull_requests(repo),
        ["repos", repo, "pulls", pull_number] => get_pull_request(repo, pull_number),
        ["repos", repo, "pulls", pull_number, "commits"] => {
            list_pull_request_commits(repo, pull_number)
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn list_pull_requests(repo: &str) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Listing pull requests of {}", repo))?;
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let repo = match Repository::load(repo, &root_dir) {
        Ok(repo) => repo,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let error_message = json!({
                "error": e.to_string()
            })
            .to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
        Err(e) => return Err(e),
    };
    let prs = repo.list_pull_requests();
    let prs = serde_json::to_string(&prs)?;
    Ok((StatusCode::Ok, Some(prs)))
}

fn get_pull_request(repo: &str, pull_number: &str) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Showing pull request {} of {}", pull_number, repo))?;
    let pull_number = match pull_number.parse::<usize>() {
        Ok(pull_number) => pull_number,
        Err(_) => {
            let error_message = json!({
                "error": "Invalid pull number: not a number."
            })
            .to_string();
            return Ok((StatusCode::BadRequest, Some(error_message)));
        }
    };
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let repo = match Repository::load(repo, &root_dir) {
        Ok(repo) => repo,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let error_message = json!({
                "error": e.to_string()
            })
            .to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
        Err(e) => return Err(e),
    };
    let pr = match repo.get_pull_request(pull_number) {
        Some(pr) => pr,
        None => {
            let error_message = json!({"error" : "Pull request not found."}).to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
    };
    let pr = serde_json::to_string(&pr)?;
    Ok((StatusCode::Ok, Some(pr)))
}

fn list_pull_request_commits(
    repo: &str,
    pull_number: &str,
) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!(
        "Listing commits of pull request {} of {}",
        pull_number, repo
    ))?;
    let pull_number = match pull_number.parse::<usize>() {
        Ok(pull_number) => pull_number,
        Err(_) => {
            let error_message = json!({
                "error": "Invalid pull number: not a number."
            })
            .to_string();
            return Ok((StatusCode::BadRequest, Some(error_message)));
        }
    };
    let current_dir = std::env::current_dir()?;
    let root_dir = &current_dir.to_string_lossy();
    let repo = match Repository::load(repo, root_dir) {
        Ok(repo) => repo,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let error_message = json!({
                "error": e.to_string()
            })
            .to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
        Err(e) => return Err(e),
    };
    let pr = match repo.get_pull_request(pull_number) {
        Some(pr) => pr,
        None => {
            let error_message = json!({"error" : "Pull request not found."}).to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
    };
    let result = match pr.list_commits(root_dir, GIT_DIR, &repo) {
        Ok(vec) => vec,
        Err(e) => {
            log("Error trying to list commits.")?;
            let error_message = json!({"error" : e.to_string()}).to_string();
            return Ok((StatusCode::BadRequest, Some(error_message)));
        }
    };

    log("Commits succesfully listed.")?;
    let result = json!(result).to_string();
    Ok((StatusCode::Ok, Some(result)))
}
