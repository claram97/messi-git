use std::io;

use serde_json::json;

use crate::{api::utils::{log::log, request::{Request, self}, status_code::StatusCode}, pull_request::{load_repo, dump_repo, PullRequestCreate, PullRequest}};

/// Handle a POST request.
pub fn handle(request: &Request) -> io::Result<(StatusCode, Option<String>)> {
    let path_splitted = request.get_path_split();
    match path_splitted[..] {
        ["repos", repo, "pulls"] => {
            let body = create_pull_request(repo, request)?;
            Ok((StatusCode::Created, Some(body)))
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn create_pull_request(repo: &str, request: &Request) -> io::Result<String> {
    log(&format!("Creating pull request in {}", repo))?;
    let curdir = std::env::current_dir()?;
    let root_dir = curdir.to_string_lossy();
    let body = &request.body;
    let pr_create: PullRequestCreate = serde_json::from_str(&body)?;
    let mut repo = load_repo(repo, &root_dir)?;
    let pr = PullRequest::new(&mut repo, pr_create)?;
    dump_repo(&repo, &root_dir)?;
    
    log(&format!("Pull request created: {:?}", pr))?;
    let pr = serde_json::to_string(&pr)?;
    Ok(pr)
}
