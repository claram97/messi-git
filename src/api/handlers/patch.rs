use std::{io, sync::Arc};

use serde_json::json;

use crate::{
    api::{
        server::{get_root_dir, Repositories},
        utils::{log::log, request::Request, status_code::StatusCode},
    },
    pull_request::PullRequestPatch,
};

/// Handle a PATCH request.
pub fn handle(
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls", pull_number] => {
            update_pull_request(repo, pull_number, request, repositories)
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn update_pull_request(
    repo: &str,
    pull_number: &str,
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!(
        "Updating pull request {} of {}",
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
    let body = &request.body;
    let pr_patch: PullRequestPatch = match serde_json::from_str(body) {
        Ok(pr_patch) => pr_patch,
        Err(e) => {
            let error_message = json!({"error": e.to_string()}).to_string();
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
            let pr = match repo.patch_pull_request(pull_number, pr_patch) {
                Ok(pr) => pr,
                Err(e) => {
                    let error_message = json!({"error": e.to_string()}).to_string();
                    return Ok((StatusCode::BadRequest, Some(error_message)));
                }
            };
            let root_dir = get_root_dir()?;
            repo.dump(&root_dir)?;
            log(&format!("Pull request updated: {:?}", &pr))?;
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
