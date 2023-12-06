use serde_json::json;

use crate::api::utils::{status_code::StatusCode, log::log};

pub fn handle(path_splitted: &[&str]) -> (StatusCode, Option<String>) {
    match path_splitted {
        ["repos", repo, "pulls"] => {
            let body = create_pull_request(repo);
            (StatusCode::Created, Some(body))
        }
        _ => (StatusCode::BadRequest, None),
    }
}

fn create_pull_request(repo: &str) -> String {
    log(&format!("Creating pull request in {}", repo));
    json!({
        "message": format!("Creando pull request en {}", repo)
    })
    .to_string()
}
