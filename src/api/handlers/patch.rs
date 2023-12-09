use std::io;

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

    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let body = &request.body;
    let pr_patch: PullRequestPatch = serde_json::from_str(body)?;
    let pull_number = pull_number.parse::<usize>().unwrap_or(0);

    let mut repo = Repository::load(repo, &root_dir)?;
    let pr = match repo.get_pull_request(pull_number) {
        Some(pr) => pr.clone(),
        None => return Ok((StatusCode::NotFound, None)),
    };

    pr.patch(&mut repo, pr_patch);
    log(&format!("Pull request updated: {:?}", &pr))?;
    repo.dump(&root_dir)?;

    let pr = serde_json::to_string(&pr)?;
    Ok((StatusCode::Ok, Some(pr)))
}
