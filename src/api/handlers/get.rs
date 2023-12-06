use crate::api::utils::status_code::StatusCode;
use serde_json::json;

pub fn handle(path_splitted: &[&str]) -> (StatusCode, Option<String>) {
    match path_splitted {
        ["repos", repo, "pulls"] => {
            let body = list_pull_requests(repo);
            (StatusCode::Ok, Some(body))
        }
        ["repos", repo, "pulls", pull_number] => {
            let body = get_pull_request(repo, pull_number);
            (StatusCode::Ok, Some(body))
        }
        ["repos", repo, "pulls", pull_number, "commits"] => {
            let body = list_pull_request_commits(repo, pull_number);
            (StatusCode::Ok, Some(body))
        }
        _ => (StatusCode::BadRequest, None),
    }
}

fn list_pull_requests(repo: &str) -> String {
    json!({
        "message": format!("Listando pull requests de {}", repo)
    })
    .to_string()
}

fn get_pull_request(repo: &str, pull_number: &str) -> String {
    json!({
        "message": format!("Mostrando pull request {} de {}", pull_number, repo)
    })
    .to_string()
}

fn list_pull_request_commits(repo: &str, pull_number: &str) -> String {
    json!({
        "message": format!("Listando commits de pull request {} de {}", pull_number, repo)
    })
    .to_string()
}
