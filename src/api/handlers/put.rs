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
    let current_dir = &current_dir.to_string_lossy().to_string();
    let repository = Repository::load(repo, current_dir)?;
    let pr = match repository.get_pull_request(pull_number) {
        Some(pr) => pr,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Pull request with the specified pull number doesn't exist.\n",
            ))
        }
    };
    let mut cloned_repo = repository.clone();
    let result = pr.merge(current_dir, GIT_DIR, &mut cloned_repo);
    cloned_repo.dump(current_dir)?;
    match result {
        Ok(_) => {
            log("Merge was successfull.")?;
        }
        Err(error) => {
            log("Unsuccessfull merge.")?;
            return Err(error);
        }
    }

    let result = json!({
        "message": format!("Listando commits de pull request {} de {}", pull_number, repo)
    })
    .to_string();
    Ok((StatusCode::Ok, Some(result)))
}
