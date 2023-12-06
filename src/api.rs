use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Data structures to represent Pull Requests and Repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Function to get a Pull Request
fn get_pull_request(repo_name: &str, pull_number: u32, state: Arc<AppState>) -> Result<String, String> {
    let pull_requests = state.pull_requests.lock().unwrap();

    if let Some(pulls) = pull_requests.get(&repo_name.to_string()) {
        // Convert pull_number to usize
        let pull_number_usize = pull_number as usize;

        if let Some(pull) = pulls.iter().find(|&pr| pr.pull_number == pull_number_usize) {
            // Serialize to JSON
            if let Ok(result) = serde_json::to_string(&pull) {
                Ok(result)
            } else {
                Err("Error converting Pull Request to JSON".to_string())
            }
        } else {
            Err("Pull Request not found".to_string())
        }
    } else {
        Err("Repository not found or no Pull Requests".to_string())
    }
}

// Function to extract the repository name and Pull Request number from the URL
fn extract_repo_and_pull_number(url: &str) -> Option<(String, u32)> {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() == 6 && parts[4] == "pulls" {
        if let Ok(pull_number) = parts[5].parse::<u32>() {
            return Some((parts[2].to_string(), pull_number));
        }
    }
    None
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
        ("GET", "/repos/{repo}/pulls/{pull_number}") => {
            if let Some((repo_name, pull_number)) = extract_repo_and_pull_number(url) {
                match get_pull_request(&repo_name, pull_number, state.clone()) {
                    Ok(result) => result,
                    Err(err) => err,
                }
            } else {
                "Invalid URL format for getting a Pull Request".to_string()
            }
        }
        _ => "Route not found".to_string(),
    }
}

// Function to extract the repository name from the URL
fn extract_repo_name(url: &str) -> Option<String> {
    url.split('/').nth(2).map(|s| s.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_pull_request_test(
        pull_number: u32,
        reviewers: Vec<&str>,
        created_at_days_ago: i64,
    ) -> PullRequest {
        let now = chrono::Utc::now();
        PullRequest {
            pull_number: pull_number.try_into().unwrap(),
            title: format!("Pull Request #{}", pull_number),
            description: "Test Pull Request".to_string(),
            source_branch: "feature-branch".to_string(),
            target_branch: "main".to_string(),
            author: "test_author".to_string(),
            created_at: (now - Duration::days(created_at_days_ago)).to_string(),
            updated_at: (now - Duration::days(created_at_days_ago)).to_string(),
            state: "open".to_string(),
            reviewers: reviewers.into_iter().map(String::from).collect(),
            closed_at: None,
        }
    }

    // Helper function to create a sample AppState with Pull Requests
    fn create_app_state_with_pull_requests(repo_name: &str, pulls: Vec<PullRequest>) -> AppState {
        let mut pull_requests_map = HashMap::new();
        pull_requests_map.insert(repo_name.to_string(), pulls);
    
        AppState {
            pull_requests: Mutex::new(pull_requests_map),
            repositories: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn test_list_pull_requests_no_repository() {
        let state = Arc::new(create_app_state_with_pull_requests("other_repo", vec![]));
        let result = list_pull_requests("owner", "repo", state, None, None, None, None);
        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some("Repository not found or no Pull Requests".to_string())
        );
    }

    #[test]
    fn test_list_pull_requests_empty_repository() {
        let state = Arc::new(create_app_state_with_pull_requests("repo", Vec::new()));
        let result = list_pull_requests("owner", "repo", state, None, None, None, None);

        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some("Repository not found or no Pull Requests".to_string())
        );
    }

    // #[test]
    // fn test_list_pull_requests_popularity_sort() {
    //     // Crear Pull Requests de prueba
    //     let pulls = vec![
    //         create_pull_request_test(1, vec!["reviewer1", "reviewer2"], 7),
    //         create_pull_request_test(2, vec!["reviewer1"], 10),
    //         create_pull_request_test(3, vec!["reviewer1", "reviewer2", "reviewer3"], 5),
    //     ];

    //     // Crear un estado con los Pull Requests de prueba
    //     let state = Arc::new(create_app_state_with_pull_requests("repo", pulls));

    //     // Listar Pull Requests ordenados por popularidad
    //     let result = list_pull_requests("owner", "repo", state, Some("popularity"), None, None, None);

    //     // Verificar que la operación tiene éxito
    //     assert!(result.is_ok());

    //     // Deserializar el resultado JSON y verificar la longitud y el orden correcto
    //     let result_json: Vec<PullRequest> = serde_json::from_str(&result.unwrap()).unwrap();
    //     assert_eq!(result_json.len(), 1);
    //     assert_eq!(result_json[0].pull_number, 3);
    // }

    #[test]
    fn test_create_pull_request() {
        let state = AppState {
            pull_requests: Mutex::new(HashMap::new()),
            repositories: Mutex::new(HashMap::new()),
        };
        let state = Arc::new(state);

        let repo_name = "test_repo";
        state.repositories.lock().unwrap().insert(
            repo_name.to_string(),
            Repository {
                name: repo_name.to_string(),
            },
        );
        let pull_request = PullRequest {
            pull_number: 0,
            title: "Test Pull Request".to_string(),
            description: "This is a test".to_string(),
            source_branch: "feature-branch".to_string(),
            target_branch: "main".to_string(),
            author: "test_author".to_string(),
            created_at: get_current_date(),
            updated_at: get_current_date(),
            state: "open".to_string(),
            reviewers: vec!["reviewer1".to_string(), "reviewer2".to_string()],
            closed_at: None,
        };

        let result = create_pull_request(repo_name, pull_request, state.clone());
        assert!(result.is_ok());

        let pull_requests = state.pull_requests.lock().unwrap();
        let repo_pulls = pull_requests.get(repo_name).unwrap();

        assert_eq!(repo_pulls.len(), 1);
        assert_eq!(repo_pulls[0].title, "Test Pull Request");
    }

    #[test]
    fn test_list_pull_requests() {
        let state = AppState {
            pull_requests: Mutex::new(HashMap::new()),
            repositories: Mutex::new(HashMap::new()),
        };
        let state = Arc::new(state);

        let repo_name = "test_repo";

        state.repositories.lock().unwrap().insert(
            repo_name.to_string(),
            Repository {
                name: repo_name.to_string(),
            },
        );

        let pull_request1 = PullRequest {
            pull_number: 1,
            title: "Pull Request 1".to_string(),
            description: "Description 1".to_string(),
            source_branch: "feature-branch-1".to_string(),
            target_branch: "main".to_string(),
            author: "Author 1".to_string(),
            created_at: get_current_date(),
            updated_at: get_current_date(),
            state: "open".to_string(),
            reviewers: vec!["reviewer1".to_string(), "reviewer2".to_string()],
            closed_at: None, 
        };
        
        let pull_request2 = PullRequest {
            pull_number: 2,
            title: "Pull Request 2".to_string(),
            description: "Description 2".to_string(),
            source_branch: "feature-branch-2".to_string(),
            target_branch: "main".to_string(),
            author: "Author 2".to_string(),
            created_at: get_current_date(),
            updated_at: get_current_date(),
            state: "open".to_string(),
            reviewers: vec!["reviewer3".to_string(), "reviewer4".to_string()],
            closed_at: Some(get_current_date()), 
        };
        
        state.pull_requests.lock().unwrap().insert(
            format!("{}/{}", repo_name, repo_name),
            vec![pull_request1, pull_request2],
        );
        let result = list_pull_requests(repo_name, repo_name, state.clone(), None, None, None, None);

        assert!(result.is_ok());

        let result_str = result.unwrap();
        let parsed_pull_requests: Vec<PullRequest> = serde_json::from_str(&result_str).unwrap();

        assert_eq!(parsed_pull_requests.len(), 2);
        assert_eq!(parsed_pull_requests[0].title, "Pull Request 2");
        assert_eq!(parsed_pull_requests[1].title, "Pull Request 1");
    }
}