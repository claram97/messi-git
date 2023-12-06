use serde_json::json;

use crate::api::utils::status_code::StatusCode;

pub fn handle(path_splitted: &[&str]) -> (StatusCode, Option<String>) {
    match path_splitted {
        ["repos", repo, "pulls", pull_number] => {
            let body = update_pull_request(repo, pull_number);
            (StatusCode::Ok, Some(body))
        }
        _ => (StatusCode::BadRequest, None),
    }
}

fn update_pull_request(repo: &str, pull_number: &str) -> String {
    json!({
        "message": format!("Actualizando pull request {} de {}", pull_number, repo)
    })
    .to_string()
}
