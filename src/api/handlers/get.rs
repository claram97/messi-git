use std::{io, sync::Arc};

use crate::{
    api::{
        server::{get_root_dir, Repositories},
        utils::{log::log, request::Request, status_code::StatusCode},
    },
    configuration::GIT_DIR,
};
use serde_json::json;

/// Handle a GET request.
pub fn handle(
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls"] => list_pull_requests(repo, repositories),
        ["repos", repo, "pulls", pull_number] => get_pull_request(repo, pull_number, repositories),
        ["repos", repo, "pulls", pull_number, "commits"] => {
            list_pull_request_commits(repo, pull_number, repositories)
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn list_pull_requests(
    repo: &str,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Listing pull requests of {}", repo))?;
    match repositories.get(repo) {
        Some(repo) => {
            let repo = match repo.lock() {
                Ok(repo) => repo,
                Err(_) => {
                    let error_message = json!({
                        "error": "Repository is locked. Try again later."
                    })
                    .to_string();
                    return Ok((StatusCode::InternalServerError, Some(error_message)));
                }
            };
            let prs = repo.list_pull_requests();
            let prs = serde_json::to_string(&prs)?;
            Ok((StatusCode::Ok, Some(prs)))
        }
        None => {
            let error_message = json!({
                "error": "Repository not found."
            })
            .to_string();
            Ok((StatusCode::NotFound, Some(error_message)))
        }
    }
}

fn get_pull_request(
    repo: &str,
    pull_number: &str,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
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
    match repositories.get(repo) {
        Some(repo) => {
            let repo = match repo.lock() {
                Ok(repo) => repo,
                Err(_) => {
                    let error_message = json!({
                        "error": "Repository is locked. Try again later."
                    })
                    .to_string();
                    return Ok((StatusCode::InternalServerError, Some(error_message)));
                }
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
        None => {
            let error_message = json!({
                "error": "Repository not found."
            })
            .to_string();
            Ok((StatusCode::NotFound, Some(error_message)))
        }
    }
}

fn list_pull_request_commits(
    repo: &str,
    pull_number: &str,
    repositories: Arc<Repositories>,
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
    match repositories.get(repo) {
        Some(repo) => {
            let repo = match repo.lock() {
                Ok(repo) => repo,
                Err(_) => {
                    let error_message = json!({
                        "error": "Repository is locked. Try again later."
                    })
                    .to_string();
                    return Ok((StatusCode::InternalServerError, Some(error_message)));
                }
            };
            let root_dir = get_root_dir()?;
            let result = match repo.list_commits_from_pull_request(pull_number, &root_dir, GIT_DIR)
            {
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
        None => {
            let error_message = json!({
                "error": "Repository not found."
            })
            .to_string();
            Ok((StatusCode::NotFound, Some(error_message)))
        }
    }
}
