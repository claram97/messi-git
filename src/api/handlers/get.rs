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
        ["repos", repo, "pulls"] => {
            let body = list_pull_requests(repo)?;
            Ok((StatusCode::Ok, Some(body)))
        }
        ["repos", repo, "pulls", pull_number] => get_pull_request(repo, pull_number),
        ["repos", repo, "pulls", pull_number, "commits"] => {
            list_pull_request_commits(repo, pull_number)
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn list_pull_requests(repo: &str) -> io::Result<String> {
    log(&format!("Listing pull requests of {}", repo))?;
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let repo = Repository::load(repo, &root_dir)?;
    let prs = repo.list_pull_requests();
    let prs = serde_json::to_string(&prs)?;
    Ok(prs)
}

fn get_pull_request(repo: &str, pull_number: &str) -> io::Result<(StatusCode, Option<String>)> {
    log(&format!("Showing pull request {} of {}", pull_number, repo))?;
    let pull_number = pull_number.parse::<usize>().unwrap_or(0);
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let repo = Repository::load(repo, &root_dir)?;
    let pr = match repo.get_pull_request(pull_number) {
        Some(pr) => pr,
        None => return Ok((StatusCode::NotFound, None)),
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
    let current_dir = std::env::current_dir()?;
    let current_dir = &current_dir.to_string_lossy().to_string();
    let repository = Repository::load(repo, current_dir)?;
    let pull_number = match pull_number.parse::<usize>() {
        Ok(pull_number) => pull_number,
        Err(_) => {
            let error_message =
                json!({"Error" : "Invalid pull number: is not a number."}).to_string();
            return Ok((StatusCode::BadRequest, Some(error_message)));
        }
    };
    let pr = match repository.get_pull_request(pull_number) {
        Some(pr) => pr,
        None => {
            let error_message = json!({"Error" : "Invalid: pull request not found."}).to_string();
            return Ok((StatusCode::NotFound, Some(error_message)));
        }
    };
    let result = pr.list_commits(current_dir, GIT_DIR, &repository);

    let vec = match result {
        Ok(vec) => {
            log("Commits succesfully listed.")?;
            vec
        }
        Err(error) => {
            log("Error trying to list commits.")?;
            return Err(error);
        }
    };

    let result = json!(vec).to_string();

    Ok((StatusCode::Ok, Some(result)))
}
