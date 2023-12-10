use std::io;

use serde_json::json;

use crate::{
    api::utils::{log::log, request::Request, status_code::StatusCode},
    configuration::GIT_DIR,
    pull_request::Repository,
};

/// Handle a PUT request.
pub fn handle(request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls", pull_number, "merge"] => merge_pull_request(repo, pull_number),
        _ => Ok((StatusCode::BadRequest, None)),
    }
}
fn merge_pull_request(repo: &str, pull_number: &str) -> io::Result<(StatusCode, Option<String>)> {
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
    let current_dir = std::env::current_dir()?;
    let root_dir = &current_dir.to_string_lossy().to_string();
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

    let result = match repo.merge_pull_request(pull_number, root_dir, GIT_DIR) {
        Ok(hash) => hash,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let error_message = json!({"error": e.to_string()}).to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        },
        Err(e) => return Err(e),
    };
    repo.dump(root_dir)?;
    log(&format!("Pull request {} merged.", pull_number))?;
    Ok((StatusCode::Ok, Some(result)))
}
