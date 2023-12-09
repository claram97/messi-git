use std::io;

use serde_json::json;

use crate::{
    api::utils::{log::log, request::Request, status_code::StatusCode},
    pull_request::{PullRequestCreate, Repository},
};

/// Handle a POST request.
pub fn handle(request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls"] => create_pull_request(repo, request),
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn create_pull_request(repo: &str, request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Creating pull request in {}", repo))?;
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
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
    let mut repo = match Repository::load(repo, &root_dir) {
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
    let pr = repo.create_pull_request(pr_create);
    repo.dump(&root_dir)?;

    log(&format!("Pull request created: {:?}", pr))?;
    let pr = serde_json::to_string(&pr)?;
    Ok((StatusCode::Ok, Some(pr)))
}
