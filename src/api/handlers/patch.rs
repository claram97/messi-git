use std::io;

use serde_json::json;

use crate::{
    api::utils::{log::log, request::Request, status_code::StatusCode},
    pull_request::{PullRequestPatch, Repository},
};

/// Handle a PATCH request.
pub fn handle(request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls", pull_number] => update_pull_request(repo, pull_number, request),
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn update_pull_request(
    repo: &str,
    pull_number: &str,
    request: &Request,
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
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let body = &request.body;
    let pr_patch: PullRequestPatch = match serde_json::from_str(body) {
        Ok(pr_patch) => pr_patch,
        Err(e) => {
            let error_message = json!({"error": e.to_string()}).to_string();
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
    let pr = match repo.patch_pull_request(pull_number, pr_patch) {
        Ok(pr) => pr,
        Err(e) => {
            let error_message = json!({"error": e.to_string()}).to_string();
            return Ok((StatusCode::BadRequest, Some(error_message)));
        }
    };
    repo.dump(&root_dir)?;
    
    log(&format!("Pull request updated: {:?}", &pr))?;
    let pr = serde_json::to_string(&pr)?;
    Ok((StatusCode::Ok, Some(pr)))
}
