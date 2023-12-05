use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Data structures to represent Pull Requests and Repositories
#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {
    pull_number: usize,
    title: String,
    description: String,
    source_branch: String,
    target_branch: String,
    author: String,
    created_at: String,
    updated_at: String,
    state: String,
    reviewers: Vec<String>,
    closed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
}

// Global state to store Pull Requests and Repositories
pub struct AppState {
    pub pull_requests: Mutex<HashMap<String, Vec<PullRequest>>>,
    pub repositories: Mutex<HashMap<String, Repository>>,
}

// API functions

// Function to create a Pull Request
fn create_pull_request(
    repo_name: &str,
    mut pull_request: PullRequest,
    state: Arc<AppState>,
) -> Result<String, String> {
    let mut pull_requests = state.pull_requests.lock().unwrap();
    let mut repositories = state.repositories.lock().unwrap();

    if let Some(repo) = repositories.get_mut(repo_name) {
        let next_pull_number = pull_requests
            .entry(repo_name.to_string())
            .or_insert_with(Vec::new)
            .len()
            + 1;

        pull_request.pull_number = next_pull_number;
        pull_request.created_at = get_current_date();
        pull_request.updated_at = pull_request.created_at.clone();

        // No need to clone each field individually, use the struct itself
        pull_request.state = "open".to_string();
        pull_request.closed_at = None;

        pull_requests
            .entry(repo_name.to_string())
            .or_insert(vec![])
            .push(pull_request);

        Ok(format!(
            "Pull Request #{} created successfully",
            next_pull_number
        ))
    } else {
        Err("Repository not found".to_string())
    }
}

// Function to get the current date and time as a string
fn get_current_date() -> String {
    use chrono::prelude::*;
    Local::now().to_string()
}

// Function to list Pull Requests
fn list_pull_requests(
    owner: &str,
    repo: &str,
    state: Arc<AppState>,
    sort: Option<&str>,
    direction: Option<&str>,
    per_page: Option<usize>,
    page: Option<usize>,
) -> Result<String, String> {
    let pull_requests = state.pull_requests.lock().unwrap();

    if let Some(pulls) = pull_requests.get(&format!("{}/{}", owner, repo)) {
        // Filter and sort Pull Requests
        let filtered_pulls: Vec<&PullRequest> = pulls
            .iter()
            .filter(|&pr| {
                // me falta la logica de filtrado (state, source branch, target branch)
                true
            })
            .collect();

        let sorted_pulls: Vec<&PullRequest> = match sort {
            Some("popularity") => {
                let max_reviewers = filtered_pulls
                    .iter()
                    .map(|pr| pr.reviewers.len())
                    .max()
                    .unwrap_or(0);
                filtered_pulls
                    .iter()
                    .filter(|&&pr| pr.reviewers.len() == max_reviewers)
                    .map(|&pr| pr)
                    .collect()
            }
            Some("long-running") => {
                let now = chrono::Utc::now();
                filtered_pulls
                    .iter()
                    .filter(|&&pr| {
                        let created_at = chrono::DateTime::parse_from_str(&pr.created_at, "%+")
                            .unwrap_or_else(|_| chrono::Utc::now().into());

                        now.signed_duration_since(created_at) > Duration::days(30)
                    })
                    .map(|&pr| pr)
                    .collect()
            }
            _ => filtered_pulls.to_vec(),
        };

        // Apply sorting direction
        let sorted_pulls: Vec<&PullRequest> = match direction {
            Some("asc") => sorted_pulls,
            Some("desc") | None => sorted_pulls.into_iter().rev().collect(),
            _ => return Err("Invalid sorting direction".to_string()),
        };

        // Paginate the results
        let start_idx = (page.unwrap_or(1) - 1) * per_page.unwrap_or(30);
        let end_idx = start_idx + per_page.unwrap_or(30);
        let paginated_pulls: Vec<&PullRequest> = sorted_pulls
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .collect();

        // Serialize to JSON
        if let Ok(result) = serde_json::to_string(&paginated_pulls) {
            Ok(result)
        } else {
            Err("Error converting Pull Requests to JSON".to_string())
        }
    } else {
        Err("Repository not found or no Pull Requests".to_string())
    }
}

// Function to handle API requests
fn handle_request(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    state: Arc<AppState>,
    git_dir: &str,
) -> String {
    match (method, url) {
        ("POST", "/repos/{repo}/pulls") => {
            if let Some(repo_name) = extract_repo_name(url) {
                if let Some(body) = body {
                    if let Ok(pull_request) = serde_json::from_slice::<PullRequest>(body) {
                        match create_pull_request(&repo_name, pull_request, state.clone()) {
                            Ok(msg) => msg,
                            Err(err) => err,
                        }
                    } else {
                        "Error parsing the request body".to_string()
                    }
                } else {
                    "Missing request body".to_string()
                }
            } else {
                "Repository name not found in the URL".to_string()
            }
        }
        ("GET", "/repos/{repo}/pulls") => {
            if let Some(repo_name) = extract_repo_name(url) {
                match list_pull_requests(
                    &repo_name,
                    &repo_name,
                    state.clone(),
                    Some("popularity"),
                    Some("asc"),
                    Some(30),
                    Some(1),
                ) {
                    Ok(result) => result,
                    Err(err) => err,
                }
            } else {
                "Repository name not found in the URL".to_string()
            }
        }
        _ => "Route not found".to_string(),
    }
}

// Function to extract the repository name from the URL
fn extract_repo_name(url: &str) -> Option<String> {
    url.split('/').nth(2).map(|s| s.to_string())
}
