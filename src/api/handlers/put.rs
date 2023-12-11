use serde_json::json;
use std::{io, sync::Arc};

use crate::{
    api::{
        server::{get_root_dir, Repositories},
        utils::{log::log, request::Request, status_code::StatusCode},
    },
    configuration::GIT_DIR,
};

/// Handle a PUT request.
pub fn handle(
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls", pull_number, "merge"] => {
            merge_pull_request(repo, pull_number, repositories)
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}
fn merge_pull_request(
    repo: &str,
    pull_number: &str,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!(
        "Merging pull request {} of {}.",
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
            let mut repo = match repo.lock() {
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
            let result = match repo.merge_pull_request(pull_number, &root_dir, GIT_DIR) {
                Ok(hash) => hash,
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => {
                        let error_message = json!({
                            "error": e.to_string()
                        })
                        .to_string();
                        return Ok((StatusCode::NotFound, Some(error_message)));
                    }
                    io::ErrorKind::InvalidInput => {
                        let error_message = json!({
                            "error": e.to_string()
                        })
                        .to_string();
                        return Ok((StatusCode::BadRequest, Some(error_message)));
                    }
                    io::ErrorKind::Interrupted => {
                        let error_message = json!({
                            "error": e.to_string()
                        })
                        .to_string();
                        return Ok((StatusCode::Conflict, Some(error_message)));
                    }
                    _ => return Err(e),
                },
            };
            repo.dump(&root_dir)?;
            log(&format!("Pull request {} merged.", pull_number))?;
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
