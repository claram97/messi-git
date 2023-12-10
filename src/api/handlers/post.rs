use std::{io, sync::Arc};

use serde_json::json;

use crate::{
    api::{
        server::{get_root_dir, Repositories},
        utils::{log::log, request::Request, status_code::StatusCode},
    },
    pull_request::PullRequestCreate,
};

/// Handle a POST request.
pub fn handle(
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls"] => create_pull_request(repo, request, repositories),
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn create_pull_request(
    repo: &str,
    request: &Request,
    repositories: Arc<Repositories>,
) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Creating pull request in {}", repo))?;
    let body = &request.body;
    let pr_create: PullRequestCreate = match serde_json::from_str(body) {
        Ok(pr_create) => pr_create,
        Err(e) => {
            let error_message = json!({
                "error": e.to_string()
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
            let pr = repo.create_pull_request(pr_create);
            let root_dir = get_root_dir()?;
            repo.dump(&root_dir)?;
            log(&format!("Pull request created: {:?}", pr))?;
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
