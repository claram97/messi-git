use std::io;

use crate::{
    api::utils::{log::log, request::Request, status_code::StatusCode},
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
            let body = list_pull_request_commits(repo, pull_number)?;
            Ok((StatusCode::Ok, Some(body)))
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

fn list_pull_request_commits(repo: &str, pull_number: &str) -> io::Result<String> {
    log(&format!(
        "Listing commits of pull request {} of {}",
        pull_number, repo
    ))?;
    let result = json!({
        "message": format!("Listando commits de pull request {} de {}", pull_number, repo)
    })
    .to_string();
    Ok(result)
}
