use std::io;

use serde_json::json;

use crate::api::utils::{status_code::StatusCode, log::log};

pub fn handle(path_splitted: &[&str]) -> io::Result<(StatusCode, Option<String>)> {
    match path_splitted {
        ["repos", repo, "pulls", pull_number, "merge"] => {
            let body = merge_pull_request(repo, pull_number)?;
            Ok((StatusCode::Ok, Some(body)))
        }
        _ => Ok((StatusCode::BadRequest, None)),
    }
}

fn merge_pull_request(repo: &str, pull_number: &str) -> io::Result<String> {
    log(&format!("Merging pull request {} of {}", pull_number, repo))?;
    let result = json!({
        "message": format!("Mergeando pull request {} de {}", pull_number, repo)
    })
    .to_string();
    Ok(result)
}
