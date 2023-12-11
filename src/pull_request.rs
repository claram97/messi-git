use crate::branch::get_branch_commit_hash;
use crate::merge;
use crate::merge::find_common_ancestor;
use crate::utils::get_branch_commit_history_until;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// Enum to represent the state of a Pull Request
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
enum PRState {
    #[default]
    Open,
    Closed,
}

/// Data structures to represent Pull Request creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestCreate {
    title: String,
    description: String,
    source_branch: String,
    target_branch: String,
}

/// Data structures to represent Pull Request patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestPatch {
    title: Option<String>,
    description: Option<String>,
    target_branch: Option<String>,
}

/// Data structures to represent Pull Request
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
    ///
    /// # Arguments
    ///
    /// * `pull_request_create` - a new PullRequestCreate to create a PullRequest
    /// * `pull_number` - the number of the PullRequest
    fn new(pull_request_create: PullRequestCreate, pull_number: usize) -> Self {
        let now = get_current_date();

        Self {
            pull_number,
            title: pull_request_create.title,
            description: pull_request_create.description,
            source_branch: pull_request_create.source_branch,
            target_branch: pull_request_create.target_branch,
            created_at: now.clone(),
            updated_at: now.clone(),
            state: PRState::Open,
            ..Default::default()
        }
    }

    /// Merges the PullRequest into the target branch
    ///
    /// # Arguments
    ///
    /// * `root_dir` - the root directory of the repository
    /// * `git_dir_name` - the name of the git directory
    /// * `repository` - the repository
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - the hash of the merge commit
    pub fn merge(&mut self, git_dir: &str) -> io::Result<String> {
        let hash =
            merge::git_merge_for_pull_request(&self.target_branch, &self.source_branch, git_dir)?;
        self.state = PRState::Closed;
        self.updated_at = get_current_date();
        self.closed_at = Some(get_current_date());
        Ok(hash)
    }

    /// Patches the PullRequest if the fields are not None
    fn patch(&mut self, pr_patch: PullRequestPatch) {
        if let Some(title) = pr_patch.title {
            self.title = title;
        }
        if let Some(description) = pr_patch.description {
            self.description = description;
        }
        if let Some(target_branch) = pr_patch.target_branch {
            self.target_branch = target_branch;
        }
        self.updated_at = get_current_date();
    }
}

/// Data structures to represent a Repository for Pull Requests
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    name: String,
    pr_count: usize,
    pull_requests: HashMap<usize, PullRequest>,
}

impl Repository {
    /// Returns a new Repository with the specified name and no Pull Requests
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the repository
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pr_count: 0,
            pull_requests: HashMap::new(),
        }
    }

    /// Creates a new PullRequest and inserts it into the repository
    ///
    /// # Arguments
    ///
    /// * `pr_create` - a PullRequestCreate to create a PullRequest
    ///
    /// # Returns
    ///
    /// * `PullRequest` - the created PullRequest
    pub fn create_pull_request(&mut self, pr_create: PullRequestCreate) -> PullRequest {
        let next_pull_number = self.pr_count + 1;
        let pull_request = PullRequest::new(pr_create, next_pull_number);
        self.insert_pull_request(&pull_request);
        pull_request
    }

    /// Patches a PullRequest with the specified pull number
    ///
    /// # Arguments
    ///
    /// * `pull_number` - the number of the PullRequest to patch
    /// * `pr_patch` - a PullRequestPatch to patch the PullRequest
    ///
    /// # Returns
    ///
    /// * `Ok(PullRequest)` - the patched and cloned PullRequest
    /// * `Err(io::Error)` - if the PullRequest doesn't exist
    pub fn patch_pull_request(
        &mut self,
        pull_number: usize,
        pr_patch: PullRequestPatch,
    ) -> io::Result<PullRequest> {
        match self.pull_requests.get_mut(&pull_number) {
            Some(pr) => {
                pr.patch(pr_patch);
                Ok(pr.clone())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Pull request with the specified pull number doesn't exist",
            )),
        }
    }

    /// Inserts a PullRequest into the repository
    fn insert_pull_request(&mut self, pull_request: &PullRequest) {
        match self
            .pull_requests
            .insert(pull_request.pull_number, pull_request.clone())
        {
            Some(_) => (),
            None => self.pr_count += 1,
        }
    }

    /// Returns the list of PullRequests in the repository sorted by pull number
    pub fn list_pull_requests(&self) -> Vec<PullRequest> {
        let mut prs: Vec<PullRequest> = self.pull_requests.values().cloned().collect();
        prs.sort_by(|a, b| a.pull_number.cmp(&b.pull_number));
        prs
    }

    /// Returns the PullRequest with the specified pull number
    ///
    /// # Arguments
    ///
    /// * `pull_number` - the number of the PullRequest to get
    ///
    /// # Returns
    ///
    /// * `Option<&PullRequest>` - the PullRequest with the specified pull number
    /// * `None` - if the PullRequest doesn't exist
    pub fn get_pull_request(&self, pull_number: usize) -> Option<&PullRequest> {
        self.pull_requests.get(&pull_number)
    }

    /// Returns the list of commits involved in the PullRequest
    ///
    /// # Arguments
    ///
    /// * `pull_number` - the number of the PullRequest to get
    /// * `root_dir` - the root directory of the repository
    /// * `git_dir_name` - the name of the git directory
    pub fn list_commits_from_pull_request(
        &self,
        pull_number: usize,
        root_dir: &str,
        git_dir_name: &str,
    ) -> io::Result<Vec<String>> {
        let git_dir = format!("{}/{}/{}", root_dir, self.name, git_dir_name);
        let pr = match self.pull_requests.get(&pull_number) {
            Some(pr) => pr,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Pull request with the specified pull number doesn't exist",
                ))
            }
        };
        let source_hash = match get_branch_commit_hash(&pr.source_branch, &git_dir) {
            Ok(hash) => hash,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Source branch doesn't exist",
                ))
            }
        };
        let target_hash = match get_branch_commit_hash(&pr.target_branch, &git_dir) {
            Ok(hash) => hash,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Target branch doesn't exist",
                ))
            }
        };
        let common_ancestor = match find_common_ancestor(&source_hash, &target_hash, &git_dir) {
            Ok(hash) => hash,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Common ancestor doesn't exist between the branches",
                ))
            }
        };
        get_branch_commit_history_until(&source_hash, &git_dir, &common_ancestor)
    }

    /// Merges a pull request identified by the specified pull number.
    ///
    /// # Arguments
    ///
    /// * `pull_number` - The unique identifier of the pull request to be merged.
    /// * `root_dir` - The root directory where repositories are stored.
    /// * `git_dir_name` - The name of the Git directory within the repository.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with the commit hash of the new merge commit on success.
    /// Returns an `io::Error` if the pull request with the specified pull number
    /// does not exist, is already closed, or if there are issues with the merge operation.
    ///
    pub fn merge_pull_request(
        &mut self,
        pull_number: usize,
        root_dir: &str,
        git_dir_name: &str,
    ) -> io::Result<String> {
        let pr = match self.pull_requests.get_mut(&pull_number) {
            Some(pr) => pr,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Pull request with the specified pull number doesn't exist",
                ))
            }
        };
        if let PRState::Closed = pr.state {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Pull request is already closed",
            ));
        }

        let git_dir = format!("{}/{}/{}", root_dir, self.name, git_dir_name);
        pr.merge(&git_dir)
    }

    #[cfg(test)]
    /// Loads a Repository from the specified root directory
    ///
    /// # Arguments
    ///
    /// * `repo` - the name of the repository to load
    /// * `root_dir` - the root directory of the repository
    ///
    /// # Returns
    ///
    /// * `Ok(Repository)` - the loaded Repository
    /// * `Err(io::Error)` - if the repository doesn't exist
    fn load(repo: &str, root_dir: &str) -> io::Result<Self> {
        let repo_dir = std::path::Path::new(root_dir).join(repo);
        if !repo_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Repository doesn't exist: {}", repo),
            ));
        }
        let filename = repo.to_string() + ".json";
        let path = std::path::Path::new(root_dir).join("prs").join(filename);
        if !path.exists() {
            return Ok(Self::new(repo));
        }
        let repo = std::fs::read_to_string(path)?;
        let repo: Self = serde_json::from_str(&repo)?;
        Ok(repo)
    }

    /// Dumps the Repository into the specified root directory
    ///
    /// # Arguments
    ///
    /// * `root_dir` - the root directory of the repository
    ///
    /// # Returns
    ///
    /// * `Ok(())` - if the dump was successful
    /// * `Err(io::Error)` - if the dump was unsuccessful
    pub fn dump(&self, root_dir: &str) -> io::Result<()> {
        let filename = root_dir.to_owned() + "/prs/" + &self.name.clone() + ".json";
        let repo = serde_json::to_string(self)?;
        let repo = repo.as_bytes();
        std::fs::write(filename, repo)
    }
}

// Function to get the current date and time as a string
fn get_current_date() -> String {
    use chrono::prelude::*;
    Local::now().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add, branch, commit, configuration::GIT_DIR_FOR_TEST};
    use std::path::Path;
    use std::{fs, io::Write};
    const TEST_SERVER_DIR: &str = "tests/pull_request/server";
    const TEST_SERVER_PRS_DIR: &str = "tests/pull_request/server/prs";

    fn create_repo_for_test(repo_name: &str) -> io::Result<()> {
        let repo = Path::new(TEST_SERVER_DIR).join(repo_name);
        if !repo.exists() {
            fs::create_dir_all(repo)?;
        }
        Ok(())
    }

    fn remove_repo_for_test(repo_name: &str) -> io::Result<()> {
        let repo = Path::new(TEST_SERVER_DIR).join(repo_name);
        if repo.exists() {
            fs::remove_dir_all(repo)?;
        }
        Ok(())
    }

    #[test]
    fn test_load_dump_repo() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_dump";
        create_repo_for_test(repo_name)?;
        let repo = Repository::load(repo_name, root_dir)?;

        repo.dump(root_dir)?;
        let loaded_repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(loaded_repo.name, repo_name);
        assert_eq!(loaded_repo.pr_count, 0);
        assert_eq!(loaded_repo.pull_requests.len(), 0);

        let repo_path = format!("{}/{}.json", TEST_SERVER_PRS_DIR, repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;

        Ok(())
    }

    #[test]
    fn test_load_repo_not_found() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_not_found";
        let loaded_repo = Repository::load(repo_name, root_dir);
        assert!(loaded_repo.is_err());
        assert!(loaded_repo.unwrap_err().kind() == io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn test_create_one_pr() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_create";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        repo.create_pull_request(pr);
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(repo.name, repo_name);
        assert_eq!(repo.pr_count, 1);
        assert_eq!(repo.pull_requests.len(), 1);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    fn test_create_many_pr() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_create_many";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        repo.create_pull_request(pr.clone());
        repo.create_pull_request(pr.clone());
        repo.create_pull_request(pr.clone());
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        assert_eq!(repo.name, repo_name);
        assert_eq!(repo.pr_count, 3);
        assert_eq!(repo.pull_requests.len(), 3);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    fn test_get_pull_request() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_get_pr";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        repo.create_pull_request(pr);
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        assert_eq!(pr.title, "title");
        assert_eq!(pr.description, "description");
        assert_eq!(pr.pull_number, 1);
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    fn test_get_pull_request_not_found() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_get_pr_not_found";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        repo.create_pull_request(pr);
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let repo = repo.get_pull_request(3);
        assert!(repo.is_none());
        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    fn test_list_prs() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_list_prs";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;

        let prs = repo.list_pull_requests();
        assert_eq!(prs.len(), 0);

        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };

        repo.create_pull_request(pr.clone());
        repo.create_pull_request(pr.clone());
        repo.create_pull_request(pr.clone());
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let prs = repo.list_pull_requests();
        assert_eq!(prs.len(), 3);

        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    fn test_patch_pr() -> io::Result<()> {
        let root_dir = TEST_SERVER_DIR;
        let repo_name = "repo_patch";
        create_repo_for_test(repo_name)?;
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: "source_branch".to_string(),
            target_branch: "target_branch".to_string(),
        };
        repo.create_pull_request(pr);
        repo.dump(root_dir)?;

        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr_patch = PullRequestPatch {
            title: Some("new title".to_string()),
            description: Some("new description".to_string()),
            target_branch: None,
        };
        repo.patch_pull_request(1, pr_patch)?;
        repo.dump(root_dir)?;

        let repo = Repository::load(repo_name, root_dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        assert_eq!(pr.title, "new title");
        assert_eq!(pr.description, "new description");
        assert_eq!(pr.target_branch, "target_branch");

        let repo_path = format!("tests/pull_request/server/prs/{}.json", repo_name);
        std::fs::remove_file(repo_path)?;
        remove_repo_for_test(repo_name)?;
        Ok(())
    }

    #[test]
    #[ignore = "run manually"]
    fn test_list_commit() -> io::Result<()> {
        let root_dir = "tests/pr_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "my_branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = repo.create_pull_request(pr);
        let commits =
            repo.list_commits_from_pull_request(pr.pull_number, root_dir, GIT_DIR_FOR_TEST);
        assert!(commits.is_ok());
        let commits = commits?;
        assert!(commits.len() == 4);

        Ok(())
    }

    #[test]
    fn test_list_commit_fails_due_to_unexisting_branch() -> io::Result<()> {
        let root_dir = "tests/pr_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = repo.create_pull_request(pr);
        let commits =
            repo.list_commits_from_pull_request(pr.pull_number, root_dir, GIT_DIR_FOR_TEST);
        assert!(commits.is_err());

        Ok(())
    }

    // #[test]
    // fn test_list_commit_fails_due_to_unexisting_repo_name() -> io::Result<()> {
    //     let root_dir = "tests/pr_list_commits";
    //     let repo_name = "repo";
    //     let mut repo = Repository::load(repo_name, root_dir)?;
    //     let pr = PullRequestCreate {
    //         title: "list commit pr".to_string(),
    //         description: "pr para testear list commits".to_string(),
    //         source_branch: "my_branch".to_string(),
    //         target_branch: "master".to_string(),
    //     };

    //     let pr = repo.create_pull_request(pr);
    //     assert!(pr.is_ok());
    //     let pr = pr?;
    //     let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
    //     assert!(commits.is_err());

    //     Ok(())
    // }

    // #[test]
    // fn test_list_commit_fails_due_to_unexisting_root_dir() -> io::Result<()> {
    //     let root_dir = "tests/pr_list_commitss";
    //     let repo_name = "repo1";
    //     let mut repo = Repository::load(repo_name, root_dir)?;
    //     let pr = PullRequestCreate {
    //         title: "list commit pr".to_string(),
    //         description: "pr para testear list commits".to_string(),
    //         source_branch: "my_branch".to_string(),
    //         target_branch: "master".to_string(),
    //     };

    //     let pr = repo.create_pull_request(pr);
    //     assert!(pr.is_ok());
    //     let pr = pr?;
    //     let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
    //     assert!(commits.is_err());

    //     Ok(())
    // }

    fn create_mock_git_dir(git_dir: &str, root_dir: &str) -> String {
        fs::create_dir_all(&git_dir).unwrap();
        let objects_dir = format!("{}/objects", git_dir);
        fs::create_dir_all(&objects_dir).unwrap();
        let refs_dir = format!("{}/refs/heads", git_dir);
        fs::create_dir_all(&refs_dir).unwrap();
        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file.write_all(b"ref: refs/heads/main").unwrap();

        let src_dir = format!("{}/src", root_dir);
        fs::create_dir_all(&src_dir).unwrap();

        let file_1_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::create(&file_1_path).unwrap();
        file.write_all(b"int main() { return 0; }").unwrap();
        let file_2_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::create(&file_2_path).unwrap();
        file.write_all(b"int hello() { return 0; }").unwrap();
        let index_file_path = format!("{}/index", git_dir);
        let _ = fs::File::create(&index_file_path).unwrap();
        add::add(&file_2_path, &index_file_path, git_dir, "", None).unwrap();
        add::add(&file_1_path, &index_file_path, git_dir, "", None).unwrap();

        let commit_message = "Initial commit";
        let commit_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();
        commit_hash
    }

    #[test]
    fn test_merge_pr_no_conflicts() -> io::Result<()> {
        if !Path::new(TEST_SERVER_PRS_DIR).exists() {
            fs::create_dir_all(TEST_SERVER_PRS_DIR)?;
        }
        let dir = TEST_SERVER_DIR;
        let repo_name = "merge";

        let root_dir = format!("{}/{}", dir, repo_name);
        if Path::new(&root_dir).exists() {
            fs::remove_dir_all(&root_dir)?;
        }

        let git_dir = Path::new(&dir).join(&repo_name).join(".mgit");
        if Path::new(&git_dir).exists() {
            fs::remove_dir_all(&git_dir)?;
        }
        std::fs::create_dir_all(&git_dir)?;
        let git_dir = git_dir.to_string_lossy().to_string();

        let main_commit_hash = create_mock_git_dir(&git_dir, &root_dir);

        let feature_branch = "feature-branch";
        branch::create_new_branch(&git_dir, feature_branch, None, &mut io::stdout())?;

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", feature_branch).as_bytes())
            .unwrap();

        let index_file_path = format!("{}/index", &git_dir);

        let file_3_path = format!("{}/src/3.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { return 0; }").unwrap();
        add::add(
            "tests/pull_request/server/merge/src/3.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_4_path = format!("{}/src/4.c", root_dir);
        let mut file = fs::File::create(&file_4_path).unwrap();
        file.write_all(b"int prueba() { return 0; }").unwrap();
        add::add(
            "tests/pull_request/server/merge/src/4.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Third commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_5_path = format!("{}/src/5.c", root_dir);
        let mut file = fs::File::create(&file_5_path).unwrap();
        file.write_all(b"int otro() { return 0; }").unwrap();
        let index_file_path = format!("{}/index", &git_dir);
        add::add(
            "tests/pull_request/server/merge/src/5.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Fourth commit";
        let commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut repo = Repository::load(repo_name, TEST_SERVER_DIR)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: feature_branch.to_string(),
            target_branch: "main".to_string(),
        };

        repo.create_pull_request(pr);
        repo.dump(&dir)?;

        let mut repo = Repository::load(repo_name, TEST_SERVER_DIR)?;
        let result = repo.merge_pull_request(1, &dir, ".mgit");

        assert!(result.is_ok());
        let merge_commit_hash = result.unwrap();
        let merge_commit = commit::is_merge_commit(&merge_commit_hash, &git_dir).unwrap();
        assert!(merge_commit);

        let merge_commit_parents = commit::get_merge_parents(&merge_commit_hash, &git_dir).unwrap();
        assert_eq!(merge_commit_parents.len(), 2);
        assert!(merge_commit_parents.contains(&main_commit_hash));
        assert!(merge_commit_parents.contains(&commit_3_hash));

        let repo_path = format!("tests/pull_request/server/merge");
        std::fs::remove_dir_all(repo_path)?;

        Ok(())
    }

    #[test]
    fn test_merge_diverging_branches_conflicts() -> io::Result<()> {
        if !Path::new(TEST_SERVER_PRS_DIR).exists() {
            fs::create_dir_all(TEST_SERVER_PRS_DIR)?;
        }
        let dir = TEST_SERVER_DIR;
        let repo_name = "merge_conflicts";

        let root_dir = format!("{}/{}", dir, repo_name);
        if Path::new(&root_dir).exists() {
            fs::remove_dir_all(&root_dir)?;
        }

        let git_dir = Path::new(&dir).join(&repo_name).join(".mgit");
        if Path::new(&git_dir).exists() {
            fs::remove_dir_all(&git_dir)?;
        }
        std::fs::create_dir_all(&git_dir)?;
        let git_dir = git_dir.to_string_lossy().to_string();

        let _ = create_mock_git_dir(&git_dir, &root_dir);

        let feature_branch = "feature-branch";
        branch::create_new_branch(&git_dir, feature_branch, None, &mut io::stdout())?;

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", feature_branch).as_bytes())
            .unwrap();

        let index_file_path = format!("{}/index", &git_dir);

        let file_4_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::create(&file_4_path).unwrap();
        file.write_all(b"int prueba() { return 0; }").unwrap();
        add::add(
            "tests/pull_request/server/merge_conflicts/src/1.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _commit_2_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/main").as_bytes())
            .unwrap();

        let file_5_path = format!("{}/src/1.c", root_dir);
        let mut file = fs::File::create(&file_5_path).unwrap();
        file.write_all(b"int otro() { return 0; }").unwrap();
        let index_file_path = format!("{}/index", &git_dir);
        add::add(
            "tests/pull_request/server/merge_conflicts/src/1.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Third commit";
        let _commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut repo = Repository::load(repo_name, TEST_SERVER_DIR)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: feature_branch.to_string(),
            target_branch: "main".to_string(),
        };

        repo.create_pull_request(pr);
        repo.dump(&dir)?;

        let mut repo = Repository::load(repo_name, TEST_SERVER_DIR)?;
        let result = repo.merge_pull_request(1, &dir, ".mgit");

        assert!(result.is_err());

        let repo_path = format!("tests/pull_request/server/merge_conflicts");
        std::fs::remove_dir_all(repo_path)?;

        Ok(())
    }
}
