use crate::api::utils::log::log;
use crate::branch::get_branch_commit_hash;
use crate::merge::find_common_ancestor;
use crate::merge::git_merge;
use crate::merge::git_merge_for_pull_request;
use crate::utils::get_branch_commit_history_until;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
enum PRState {
    #[default]
    Open,
    Updated,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestCreate {
    title: String,
    description: String,
    source_branch: String,
    target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestPatch {
    title: Option<String>,
    description: Option<String>,
    target_branch: Option<String>,
}

// Data structures to represent Pull Requests and Repositories
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pull_number: usize,
    title: String,
    description: String,
    source_branch: String,
    target_branch: String,
    author: String,
    created_at: String,
    updated_at: String,
    state: PRState,
    reviewers: Vec<String>,
    closed_at: Option<String>,
}

impl PullRequest {
    /// Returns a new PullRequest
    pub fn new(repo: &mut Repository, pull_request_create: PullRequestCreate) -> io::Result<Self> {
        let now = get_current_date();
        let next_pull_number = repo.pr_count + 1;

        let pr = Self {
            pull_number: next_pull_number,
            title: pull_request_create.title,
            description: pull_request_create.description,
            source_branch: pull_request_create.source_branch,
            target_branch: pull_request_create.target_branch,
            created_at: now.clone(),
            updated_at: now.clone(),
            state: PRState::Open,
            ..Default::default()
        };
        repo.insert_pull_request(&pr);
        Ok(pr)
    }

    pub fn list_commits(
        &self,
        root_dir: &str,
        git_dir_name: &str,
        repository: &mut Repository,
    ) -> io::Result<Vec<String>> {
        let git_dir = format!("{}/{}/{}", root_dir, repository.name, git_dir_name);
        let source_hash = get_branch_commit_hash(&self.source_branch, &git_dir)?;
        let target_hash = get_branch_commit_hash(&self.target_branch, &git_dir)?;
        let common_ancestor = find_common_ancestor(&source_hash, &target_hash, &git_dir)?;
        get_branch_commit_history_until(&source_hash, &git_dir, &common_ancestor)
    }

    /// Returns a new PullRequest with the updated fields
    pub fn patch(&self, repo: &mut Repository, pr_patch: PullRequestPatch) -> Self {
        let mut pr = self.clone();
        if let Some(title) = pr_patch.title {
            pr.title = title;
        }
        if let Some(description) = pr_patch.description {
            pr.description = description;
        }
        if let Some(target_branch) = pr_patch.target_branch {
            pr.target_branch = target_branch;
        }
        let now = get_current_date();
        pr.updated_at = now.clone();
        repo.insert_pull_request(&pr);
        pr
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    name: String,
    pr_count: usize,
    pull_requests: HashMap<usize, PullRequest>,
}

impl Repository {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pr_count: 0,
            pull_requests: HashMap::new(),
        }
    }

    pub fn insert_pull_request(&mut self, pull_request: &PullRequest) {
        match self
            .pull_requests
            .insert(pull_request.pull_number, pull_request.clone())
        {
            Some(_) => (),
            None => self.pr_count += 1,
        }
    }

    pub fn list_pull_requests(&self) -> Vec<PullRequest> {
        let mut prs: Vec<PullRequest> = self.pull_requests.values().cloned().collect();
        prs.sort_by(|a, b| a.pull_number.cmp(&b.pull_number));
        prs
    }

    pub fn get_pull_request(&self, pull_number: usize) -> Option<&PullRequest> {
        self.pull_requests.get(&pull_number)
    }

    pub fn load(repo: &str, root_dir: &str) -> io::Result<Self> {
        let filename = repo.to_string() + ".json";
        let path = Path::new(root_dir).join("prs").join(&filename);
        if !path.exists() {
            return Ok(Self::new(repo));
        }
        let repo = std::fs::read_to_string(path)?;
        let repo: Self = serde_json::from_str(&repo)?;
        Ok(repo)
    }

    pub fn dump(&self, root_dir: &str) -> io::Result<()> {
        let filename = root_dir.to_owned() + "/prs/" + &self.name.clone() + ".json";
        let repo = serde_json::to_string(self)?;
        let repo = repo.as_bytes();
        std::fs::write(filename, repo)
    }
}

// // Global state to store Pull Requests and Repositories
pub struct AppState {
    pull_requests: Mutex<HashMap<String, Vec<PullRequest>>>,
    repositories: Mutex<HashMap<String, Repository>>,
}

// API functions

// Function to create a Pull Request
// fn create_pull_request(
//     repo_name: &str,
//     // repo: &mut Repository,
//     mut pull_request: PullRequest,
//     state: Arc<AppState>,
// ) -> Result<String, String> {
//     let mut pull_requests = state.pull_requests.lock().unwrap();
//     let mut repositories = state.repositories.lock().unwrap();

//     if let Some(repo) = repositories.get_mut(repo_name) {
//         let next_pull_number = pull_requests
//             .entry(repo_name.to_string())
//             .or_insert_with(Vec::new)
//             .len()
//             + 1;

//         pull_request.pull_number = next_pull_number;
//         pull_request.created_at = get_current_date();
//         pull_request.updated_at = pull_request.created_at.clone();
//         pull_request.state = PRState::Open;
//         pull_request.closed_at = None;

//         pull_requests
//             .entry(repo_name.to_string())
//             .or_insert(vec![])
//             .push(pull_request);

//         Ok(format!(
//             "Pull Request #{} created successfully",
//             next_pull_number
//         ))
//     } else {
//         Err("Repository not found".to_string())
//     }
// }

// Function to get the current date and time as a string
fn get_current_date() -> String {
    use chrono::prelude::*;
    Local::now().to_string()
}

fn list_pull_requests(repo: &str, state: Arc<AppState>) -> Result<String, String> {
    let pull_requests = state.pull_requests.lock().unwrap();

    if let Some(pulls) = pull_requests.get(repo) {
        let pulls_clone: Vec<&PullRequest> = pulls.iter().collect();

        if let Ok(result) = serde_json::to_string(&pulls_clone) {
            Ok(result)
        } else {
            Err("Error converting Pull Requests to JSON".to_string())
        }
    } else {
        Err("Repository not found or no Pull Requests".to_string())
    }
}

// Function to get a Pull Request
fn get_pull_request(
    repo_name: &str,
    pull_number: u32,
    state: Arc<AppState>,
) -> Result<String, String> {
    let pull_requests = state.pull_requests.lock().unwrap();

    if let Some(pulls) = pull_requests.get(&repo_name.to_string()) {
        let pull_number_usize = pull_number as usize;

        if let Some(pull) = pulls.iter().find(|&pr| pr.pull_number == pull_number_usize) {
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

fn list_commits(repo_name: &str, pull_number: u32, state: Arc<AppState>) -> Result<String, String> {
    let pull_requests = match state.pull_requests.lock() {
        Ok(pull_requests) => pull_requests,
        Err(error) => {
            return Err(format!("Error acquiring Mutex lock: {}", error));
        }
    };

    if let Some(pulls) = pull_requests.get(&repo_name.to_string()) {
        let pull_number_usize = pull_number as usize;

        if let Some(pull) = pulls.iter().find(|&pr| pr.pull_number == pull_number_usize) {
            let source_branch = &pull.source_branch;
            let target_branch = &pull.target_branch;

            let git_repo_dir = get_git_repo_dir(repo_name, &state).ok_or_else(|| {
                "Repository not found or no path specified for the repository".to_string()
            })?;

            let git_dir = git_repo_dir.to_string_lossy().to_string();
            let source_branch_hash = match get_branch_commit_hash(source_branch, &git_dir) {
                Ok(hash) => hash,
                Err(error) => return Err(error.to_string()),
            };
            println!(
                "Commit hash for source branch '{}': {}",
                source_branch, source_branch_hash
            );

            let target_branch_hash = match get_branch_commit_hash(target_branch, &git_dir) {
                Ok(hash) => hash,
                Err(error) => return Err(error.to_string()),
            };
            println!(
                "Commit hash for target branch '{}': {}",
                target_branch, target_branch_hash
            );

            let common_ancestor =
                match find_common_ancestor(&source_branch_hash, &target_branch_hash, &git_dir) {
                    Ok(ancestor) => ancestor,
                    Err(error) => return Err(error.to_string()),
                };

            let commit_history = match get_branch_commit_history_until(
                &source_branch_hash,
                &git_dir,
                &common_ancestor,
            ) {
                Ok(commit) => commit,
                Err(error) => return Err(error.to_string()),
            };

            match serde_json::to_string(&commit_history) {
                Ok(string) => return Ok(string),
                Err(error) => return Err(error.to_string()),
            };
        } else {
            return Err("Pull Request not found".to_string());
        }
    } else {
        return Err("Repository not found or no Pull Requests".to_string());
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

// Function to list commits in a Pull Request
// fn get_git_repo_dir(repo_name: &str, state: &Arc<AppState>) -> Option<String> {
//     let repositories = state.repositories.lock().unwrap();

//     repositories.get(repo_name).cloned()
// }

// fn list_commits(
//     repo_name: &str,
//     pull_number: u32,
//     state: Arc<AppState>,
// ) -> Result<String, String> {
//     let pull_requests = state.pull_requests.lock().unwrap();

//     if let Some(pulls) = pull_requests.get(&repo_name.to_string()) {
//         // Convert pull_number to usize
//         let pull_number_usize = pull_number as usize;

//         if let Some(pull) = pulls.iter().find(|&pr| pr.pull_number == pull_number_usize) {
//             // Extract source_branch and target_branch from the Pull Request
//             let source_branch = &pull.source_branch;
//             let target_branch = &pull.target_branch;

//             // Get commit hash for source_branch
//             let git_repo_dir = get_git_repo_dir(repo_name, &state).ok_or_else(|| {
//                 "Repository not found or no path specified for the repository".to_string()
//             })?;
//             let source_branch_hash = get_branch_commit_hash(source_branch, &git_repo_dir)?;
//             println!("Commit hash for source branch '{}': {}", source_branch, source_branch_hash);

//             // Get commit hash for target_branch
//             let target_branch_hash = get_branch_commit_hash(target_branch, &git_repo_dir)?;
//             println!("Commit hash for target branch '{}': {}", target_branch, target_branch_hash);

//             // Check if log function succeeded
//             if let Ok(log_iter) = log_result {
//                 // Collect commits from the iterator
//                 let commits: Vec<Commit> = log_iter
//                     .map(|log_entry| Commit {
//                         sha: log_entry.hash,
//                         message: log_entry.message,
//                     })
//                     .collect();

//                 // Serialize to JSON
//                 if let Ok(result) = serde_json::to_string(&commits) {
//                     return Ok(result);
//                 } else {
//                     return Err("Error converting Commits to JSON".to_string());
//                 }
//             } else {
//                 return Err(format!(
//                     "Error retrieving commits for pull request {}",
//                     pull_number
//                 ));
//             }
//         } else {
//             return Err("Pull Request not found".to_string());
//         }
//     } else {
//         return Err("Repository not found or no Pull Requests".to_string());
//     }
// }

fn modify_pull_request(
    repo_name: &str,
    pull_number: u32,
    updated_data: PullRequest,
    state: Arc<AppState>,
) -> Result<String, String> {
    let mut pull_requests = state.pull_requests.lock().unwrap();

    if let Some(pulls) = pull_requests.get_mut(&repo_name.to_string()) {
        let pull_number_usize = pull_number as usize;

        if let Some(pull) = pulls
            .iter_mut()
            .find(|pr| pr.pull_number == pull_number_usize)
        {
            pull.title = updated_data.title;
            pull.description = updated_data.description;
            pull.source_branch = updated_data.source_branch;
            pull.target_branch = updated_data.target_branch;
            pull.author = updated_data.author;
            pull.created_at = updated_data.created_at;
            pull.updated_at = updated_data.updated_at;
            pull.state = updated_data.state;
            pull.reviewers = updated_data.reviewers;
            pull.closed_at = updated_data.closed_at;

            if let Ok(result) = serde_json::to_string(pull) {
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

fn get_git_repo_dir(repo_name: &str, state: &Arc<AppState>) -> Option<PathBuf> {
    // Lock the Mutex to access the repositories in the state
    let repositories = state.repositories.lock().unwrap();

    // Attempt to retrieve the repository corresponding to the provided name
    if let Some(repo) = repositories.get(repo_name) {
        let root_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        // If the repository is found, construct the path to the repository directory
        // Assuming repositories are located in a subdirectory named "repos"
        let repo_dir = Path::new(&root_dir).join("repos").join(&repo_name);

        // Return Some with the path to the repository directory
        Some(repo_dir)
    } else {
        // If the repository is not found, return None
        None
    }
}
//dejo los comentarios en espaniol por si es mas facil entender hasta q este
fn merge_pull_request(
    repo_name: &str,
    pull_number: u32,
    state: Arc<AppState>,
) -> Result<String, String> {
    // Obtener el repositorio desde el estado
    let repositories = state.repositories.lock().unwrap();
    if let Some(repo) = repositories.get(repo_name) {
        // Obtener el Pull Request desde el estado
        let pull_requests = state.pull_requests.lock().unwrap();
        if let Some(pull) = pull_requests.get(repo_name).and_then(|prs| {
            prs.iter()
                .find(|&pr| pr.pull_number == pull_number as usize)
        }) {
            // Suponiendo que puedes extraer la información necesaria del Pull Request
            let our_branch = &pull.target_branch;
            let their_branch = &pull.source_branch;
            let git_repo_dir = get_git_repo_dir(repo_name, &state).ok_or_else(|| {
                format!(
                    "Repository '{}' not found or no path specified for the repository",
                    repo_name
                )
            })?;

            match git_merge_for_pull_request(
                our_branch,
                their_branch,
                &git_repo_dir.to_string_lossy(),
            ) {
                Ok(hash) => Ok(format!(
                    "Pull Request #{} merged successfully. Commit hash: {}",
                    pull_number, hash
                )),
                Err(err) => Err(format!("Error merging Pull Request: {}", err)),
            }
        } else {
            Err(format!("Pull Request #{} not found", pull_number))
        }
    } else {
        Err(format!("Repository '{}' not found", repo_name))
    }
}

// Function to handle API requests
// fn handle_request(
//     method: &str,
//     url: &str,
//     body: Option<&[u8]>,
//     state: Arc<AppState>,
//     git_dir: &str,
// ) -> String {
//     match (method, url) {
//         ("POST", "/repos/{repo}/pulls") => {
//             if let Some(repo_name) = extract_repo_name(url) {
//                 if let Some(body) = body {
//                     if let Ok(pull_request) = serde_json::from_slice::<PullRequest>(body) {
//                         match create_pull_request(&repo_name, pull_request, state.clone()) {
//                             Ok(msg) => msg,
//                             Err(err) => err,
//                         }
//                     } else {
//                         "Error parsing the request body".to_string()
//                     }
//                 } else {
//                     "Missing request body".to_string()
//                 }
//             } else {
//                 "Repository name not found in the URL".to_string()
//             }
//         }
//         ("GET", "/repos/{repo}/pulls") => {
//             if let Some(repo_name) = extract_repo_name(url) {
//                 match list_pull_requests(&repo_name, state.clone()) {
//                     Ok(result) => result,
//                     Err(err) => err,
//                 }
//             } else {
//                 "Repository name not found in the URL".to_string()
//             }
//         }
//         ("GET", "/repos/{repo}/pulls/{pull_number}") => {
//             if let Some((repo_name, pull_number)) = extract_repo_and_pull_number(url) {
//                 match get_pull_request(&repo_name, pull_number, state.clone()) {
//                     Ok(result) => result,
//                     Err(err) => err,
//                 }
//             } else {
//                 "Invalid URL format for getting a Pull Request".to_string()
//             }
//         }
//         ("PATCH", "/repos/{repo}/pulls/{pull_number}") => {
//             if let Some((repo_name, pull_number)) = extract_repo_and_pull_number(url) {
//                 if let Some(body) = body {
//                     if let Ok(updated_data) = serde_json::from_slice::<PullRequest>(body) {
//                         match modify_pull_request(
//                             &repo_name,
//                             pull_number,
//                             updated_data,
//                             state.clone(),
//                         ) {
//                             Ok(msg) => msg,
//                             Err(err) => err,
//                         }
//                     } else {
//                         "Error parsing the request body".to_string()
//                     }
//                 } else {
//                     "Missing request body".to_string()
//                 }
//             } else {
//                 "Invalid URL format for modifying a Pull Request".to_string()
//             }
//         }
//         ("PUT", "/repos/{repo}/pulls/{pull_number}/merge") => {
//             if let Some((repo_name, pull_number)) = extract_repo_and_pull_number(url) {
//                 match merge_pull_request(&repo_name, pull_number, state.clone()) {
//                     Ok(msg) => msg,
//                     Err(err) => err,
//                 }
//             } else {
//                 "Invalid URL format for merging a Pull Request".to_string()
//             }
//         }

//         // ("GET", "/repos/{repo}/pulls/{pull_number}/commits") => {
//         //     if let Some((repo_name, pull_number)) = extract_repo_and_pull_number(url) {
//         //         match list_commits(&repo_name, pull_number, state.clone()) {
//         //             Ok(result) => result,
//         //             Err(err) => err,
//         //         }
//         //     } else {
//         //         "Invalid URL format for listing commits in a Pull Request".to_string()
//         //     }
//         // }
//         _ => "Route not found".to_string(),
//     }
// }

// Function to extract the repository name from the URL
// fn extract_repo_name(url: &str) -> Option<String> {
//     url.split('/').nth(2).map(|s| s.to_string())
// }

#[cfg(test)]
mod tests {
    use crate::configuration::GIT_DIR_FOR_TEST;

    use super::*;
    use std::collections::HashMap;
    // use std::sync::{Arc, Mutex};

    // fn create_pull_request_test(
    //     pull_number: u32,
    //     reviewers: Vec<&str>,
    //     created_at_days_ago: i64,
    // ) -> PullRequest {
    //     let now = chrono::Utc::now();
    //     PullRequest {
    //         pull_number: pull_number.try_into().unwrap(),
    //         title: format!("Pull Request #{}", pull_number),
    //         description: "Test Pull Request".to_string(),
    //         source_branch: "feature-branch".to_string(),
    //         target_branch: "main".to_string(),
    //         author: "test_author".to_string(),
    //         created_at: (now - Duration::days(created_at_days_ago)).to_string(),
    //         updated_at: (now - Duration::days(created_at_days_ago)).to_string(),
    //         state: PRState::Open,
    //         reviewers: reviewers.into_iter().map(String::from).collect(),
    //         closed_at: None,
    //     }
    // }

    // Helper function to create a sample AppState with Pull Requests
    // fn create_app_state_with_pull_requests(repo_name: &str, pulls: Vec<PullRequest>) -> AppState {
    //     let mut pull_requests_map = HashMap::new();
    //     pull_requests_map.insert(repo_name.to_string(), pulls);

    //     AppState {
    //         pull_requests: Mutex::new(pull_requests_map),
    //         repositories: Mutex::new(HashMap::new()),
    //     }
    // }

    // #[test]
    // fn test_modify_pull_request_success() {
    //     let repo_name = "test_repo";
    //     let pull_number = 1;
    //     let state = Arc::new(create_app_state_with_pull_requests(
    //         repo_name,
    //         vec![create_pull_request_test(pull_number, vec![], 7)],
    //     ));

    //     let updated_data = PullRequest {
    //         pull_number: pull_number.try_into().unwrap(),
    //         title: "Updated Title".to_string(),
    //         description: "Updated Description".to_string(),
    //         source_branch: "updated-source".to_string(),
    //         target_branch: "updated-target".to_string(),
    //         author: "updated-author".to_string(),
    //         created_at: get_current_date(),
    //         updated_at: get_current_date(),
    //         state: "updated-state".to_string(),
    //         reviewers: vec!["updated-reviewer".to_string()],
    //         closed_at: Some(get_current_date()),
    //     };

    //     let result = modify_pull_request(repo_name, pull_number, updated_data, state.clone());

    //     assert!(result.is_ok());
    //     let result_json: PullRequest = serde_json::from_str(&result.unwrap()).unwrap();
    //     assert_eq!(result_json.title, "Updated Title");
    //     assert_eq!(result_json.description, "Updated Description");
    //     assert_eq!(result_json.source_branch, "updated-source");
    //     assert_eq!(result_json.target_branch, "updated-target");
    //     assert_eq!(result_json.author, "updated-author");
    //     assert_eq!(result_json.state, "updated-state");
    // }

    // #[test]
    // fn test_modify_pull_request_not_found_pull() {
    //     let state = Arc::new(create_app_state_with_pull_requests("test_repo", vec![]));

    //     let updated_data = PullRequest {
    //         pull_number: 1,
    //         title: "Updated Title".to_string(),
    //         description: "Updated Description".to_string(),
    //         source_branch: "updated-source".to_string(),
    //         target_branch: "updated-target".to_string(),
    //         author: "updated-author".to_string(),
    //         created_at: get_current_date(),
    //         updated_at: get_current_date(),
    //         state: "updated-state".to_string(),
    //         reviewers: vec!["updated-reviewer".to_string()],
    //         closed_at: Some(get_current_date()),
    //     };

    //     let result = modify_pull_request("test_repo", 1, updated_data, state.clone());

    //     assert!(result.is_err());
    //     assert_eq!(result.err(), Some("Pull Request not found".to_string()));
    // }

    // #[test]
    // fn test_get_pull_request_success() {
    //     let state = create_app_state_with_pull_requests(
    //         "test_repo",
    //         vec![
    //             create_pull_request_test(1, vec!["reviewer1", "reviewer2"], 7),
    //             create_pull_request_test(2, vec!["reviewer1"], 10),
    //         ],
    //     );
    //     let state = Arc::new(state);
    //     let result = get_pull_request("test_repo", 1, state.clone());
    //     assert!(result.is_ok());
    //     let result_json: PullRequest = serde_json::from_str(&result.unwrap()).unwrap();
    //     assert_eq!(result_json.pull_number, 1);
    //     assert_eq!(result_json.title, "Pull Request #1");
    // }

    // #[test]
    // fn test_get_pull_request_not_found_pull() {
    //     let state = create_app_state_with_pull_requests(
    //         "test_repo",
    //         vec![
    //             create_pull_request_test(1, vec!["reviewer1", "reviewer2"], 7),
    //             create_pull_request_test(2, vec!["reviewer1"], 10),
    //         ],
    //     );
    //     let state = Arc::new(state);
    //     let result = get_pull_request("test_repo", 3, state.clone());
    //     assert!(result.is_err());
    //     assert_eq!(result.err(), Some("Pull Request not found".to_string()));
    // }

    // #[test]
    // fn test_get_pull_request_not_found_repo() {
    //     let state = create_app_state_with_pull_requests("test_repo", vec![]);
    //     let state = Arc::new(state);
    //     let result = get_pull_request("nonexistent_repo", 1, state.clone());
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.err(),
    //         Some("Repository not found or no Pull Requests".to_string())
    //     );
    // }

    // #[test]
    // fn test_list_pull_requests_no_repository() {
    //     let state = Arc::new(create_app_state_with_pull_requests("other_repo", vec![]));
    //     let result = list_pull_requests("repo", state);
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.err(),
    //         Some("Repository not found or no Pull Requests".to_string())
    //     );
    // }

    // #[test]
    // fn test_list_pull_requests_empty_repository() {
    //     let state = Arc::new(create_app_state_with_pull_requests(
    //         "repo_empty",
    //         Vec::new(),
    //     ));
    //     let result = list_pull_requests("repo_empty", state.clone());

    //     assert!(result.is_ok());
    //     let result_str = result.unwrap();
    //     let parsed_pull_requests: Vec<PullRequest> = serde_json::from_str(&result_str).unwrap();

    //     assert_eq!(parsed_pull_requests.len(), 0);
    // }

    // #[test]
    // fn test_list_pull_requests_success() {
    //     let repo_name = "test_repo";
    //     let pull_request1 = create_pull_request_test(1, vec!["reviewer1", "reviewer2"], 7);
    //     let pull_request2 = create_pull_request_test(2, vec!["reviewer1"], 10);
    //     let state =
    //         create_app_state_with_pull_requests(repo_name, vec![pull_request1, pull_request2]);
    //     let result = list_pull_requests(repo_name, Arc::new(state));

    //     assert!(result.is_ok());
    //     let result_str = result.unwrap();
    //     assert!(result_str.contains("Pull Request #1"));
    //     assert!(result_str.contains("Pull Request #2"));
    // }

    // #[test]
    // fn test_create_pull_request() {
    //     let state = AppState {
    //         pull_requests: Mutex::new(HashMap::new()),
    //         repositories: Mutex::new(HashMap::new()),
    //     };
    //     let state = Arc::new(state);

    //     let repo_name = "test_repo";
    //     state.repositories.lock().unwrap().insert(
    //         repo_name.to_string(),
    //         Repository {
    //             name: repo_name.to_string(),
    //         },
    //     );
    //     let pull_request = PullRequest {
    //         pull_number: 0,
    //         title: "Test Pull Request".to_string(),
    //         description: "This is a test".to_string(),
    //         source_branch: "feature-branch".to_string(),
    //         target_branch: "main".to_string(),
    //         author: "test_author".to_string(),
    //         created_at: get_current_date(),
    //         updated_at: get_current_date(),
    //         state: "open".to_string(),
    //         reviewers: vec!["reviewer1".to_string(), "reviewer2".to_string()],
    //         closed_at: None,
    //     };

    //     let result = create_pull_request(repo_name, pull_request, state.clone());
    //     assert!(result.is_ok());

    //     let pull_requests = state.pull_requests.lock().unwrap();
    //     let repo_pulls = pull_requests.get(repo_name).unwrap();

    //     assert_eq!(repo_pulls.len(), 1);
    //     assert_eq!(repo_pulls[0].title, "Test Pull Request");
    // }

    #[test]
    fn test_load_dump_repo() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_dump";
        let repo = Repository::load(repo_name, root_dir)?;

        repo.dump(root_dir)?;
        let loaded_repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(loaded_repo.name, repo_name);
        assert_eq!(loaded_repo.pr_count, 0);
        assert_eq!(loaded_repo.pull_requests.len(), 0);

        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;

        let repo_name = "not_exist";
        let loaded_repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(loaded_repo.name, repo_name);
        assert_eq!(loaded_repo.pr_count, 0);
        assert_eq!(loaded_repo.pull_requests.len(), 0);

        Ok(())
    }

    #[test]
    fn test_create_one_pr() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_create";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        PullRequest::new(&mut repo, pr)?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(repo.name, repo_name);
        assert_eq!(repo.pr_count, 1);
        assert_eq!(repo.pull_requests.len(), 1);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_create_many_pr() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_create_many";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        PullRequest::new(&mut repo, pr.clone())?;
        PullRequest::new(&mut repo, pr.clone())?;
        PullRequest::new(&mut repo, pr.clone())?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(repo.name, repo_name);
        assert_eq!(repo.pr_count, 3);
        assert_eq!(repo.pull_requests.len(), 3);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_get_pull_request() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_get_pr";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        PullRequest::new(&mut repo, pr.clone())?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        assert_eq!(pr.title, "title");
        assert_eq!(pr.description, "description");
        assert_eq!(pr.pull_number, 1);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_get_pull_request_not_found() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_get_pr_not_found";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        PullRequest::new(&mut repo, pr.clone())?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let repo = repo.get_pull_request(3);
        assert!(repo.is_none());
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_list_prs() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_list_prs";
        let mut repo = Repository::load(repo_name, root_dir)?;

        let prs = repo.list_pull_requests();
        assert_eq!(prs.len(), 0);

        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        PullRequest::new(&mut repo, pr.clone())?;
        PullRequest::new(&mut repo, pr.clone())?;
        PullRequest::new(&mut repo, pr.clone())?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let prs = repo.list_pull_requests();
        assert_eq!(prs.len(), 3);

        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_patch_pr() -> io::Result<()> {
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let root_dir = "tests/pull_request/server";
        let repo_name = "repo_patch";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        PullRequest::new(&mut repo, pr)?;
        repo.dump(root_dir)?;

        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = repo.get_pull_request(1).unwrap();
        let pr_patch = PullRequestPatch {
            title: Some("new title".to_string()),
            description: Some("new description".to_string()),
            target_branch: None,
        };

        pr.clone().patch(&mut repo, pr_patch);

        repo.dump(root_dir)?;
        let repo = Repository::load(repo_name, root_dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        assert_eq!(pr.title, "new title");
        assert_eq!(pr.description, "new description");
        assert_eq!(pr.target_branch, "target_branch");

        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        Ok(())
    }

    #[test]
    fn test_list_commit() -> io::Result<()> {
        let root_dir = "tests/test_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "my_branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr)?;

        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_ok());
        let commits = commits?;
        dbg!("{:?}", commits);

        Ok(())
    }

    #[test]
    fn test_list_commit_fails_due_to_unexisting_branch() -> io::Result<()> {
        let root_dir = "tests/test_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr);
        assert!(pr.is_ok());
        let pr = pr?;
        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_err());

        Ok(())
    }

    #[test]
    fn test_list_commit_fails_due_to_unexisting_repo_name() -> io::Result<()> {
        let root_dir = "tests/test_list_commits";
        let repo_name = "repo";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "my_branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr);
        assert!(pr.is_ok());
        let pr = pr?;
        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_err());

        Ok(())
    }

    #[test]
    fn test_list_commit_fails_due_to_unexisting_root_dir() -> io::Result<()> {
        let root_dir = "tests/test_list_commitss";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "my_branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr);
        assert!(pr.is_ok());
        let pr = pr?;
        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_err());

        Ok(())
    }
}
