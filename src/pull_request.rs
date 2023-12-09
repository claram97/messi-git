use crate::branch::get_branch_commit_hash;
use crate::merge;
use crate::merge::find_common_ancestor;
use crate::utils::get_branch_commit_history_until;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;

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

    pub fn merge(
        &self,
        root_dir: &str,
        git_dir_name: &str,
        repository: &mut Repository,
    ) -> io::Result<String> {
        let git_dir = format!("{}/{}/{}", root_dir, repository.name, git_dir_name);
        let hash =
            merge::git_merge_for_pull_request(&self.target_branch, &self.source_branch, &git_dir)?;
        let mut pr = self.clone();
        pr.state = PRState::Closed;
        pr.updated_at = get_current_date();
        pr.closed_at = Some(get_current_date());
        repository.insert_pull_request(&pr);
        Ok(hash)
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

// Function to get the current date and time as a string
fn get_current_date() -> String {
    use chrono::prelude::*;
    Local::now().to_string()
}

#[cfg(test)]
mod tests {
    use crate::{add, branch, commit, configuration::GIT_DIR_FOR_TEST};

    use super::*;
    use std::{fs, io::Write};

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
        assert!(commits.len() == 4);

        Ok(())
    }

    #[test]
    fn test_list_commit_2() -> io::Result<()> {
        let root_dir = "tests/test_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "new_branch".to_string(),
            target_branch: "master".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr)?;

        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_ok());
        let commits = commits?;
        assert!(commits.len() == 6);

        Ok(())
    }

    #[test]
    fn test_list_commit_3() -> io::Result<()> {
        let root_dir = "tests/test_list_commits";
        let repo_name = "repo1";
        let mut repo = Repository::load(repo_name, root_dir)?;
        let pr = PullRequestCreate {
            title: "list commit pr".to_string(),
            description: "pr para testear list commits".to_string(),
            source_branch: "new_branch".to_string(),
            target_branch: "my_branch".to_string(),
        };

        let pr = PullRequest::new(&mut repo, pr)?;

        let commits = pr.list_commits(root_dir, GIT_DIR_FOR_TEST, &mut repo);
        assert!(commits.is_ok());
        let commits = commits?;
        assert!(commits.len() == 2);

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
        if Path::new("tests/pull_request/server/prs").exists() {
            fs::remove_dir_all("tests/pull_request/server/prs")?;
        }
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let dir = "tests/pull_request/server";
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

        let mut repo = Repository::load(repo_name, &root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: feature_branch.to_string(),
            target_branch: "main".to_string(),
        };

        PullRequest::new(&mut repo, pr)?;
        repo.dump(&dir)?;

        let repo = Repository::load(repo_name, &dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        let mut repo = Repository::load(repo_name, &dir)?;
        let result = pr.merge(&dir, ".mgit", &mut repo);

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
    fn test_merge_pr_conflicts() -> io::Result<()> {
        if Path::new("tests/pull_request/server/prs").exists() {
            fs::remove_dir_all("tests/pull_request/server/prs")?;
        }
        std::fs::create_dir_all("tests/pull_request/server/prs")?;
        let dir = "tests/pull_request/server";
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

        let file_3_path = format!("{}/src/2.c", root_dir);
        let mut file = fs::File::create(&file_3_path).unwrap();
        file.write_all(b"int bye() { return 0; }").unwrap();
        add::add(
            "tests/pull_request/server/merge_conflicts/src/2.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

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

        let commit_message = "Third commit";
        let commit_2_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/main").as_bytes())
            .unwrap();

        let file_5_path = format!("{}/src/5.c", root_dir);
        let mut file = fs::File::create(&file_5_path).unwrap();
        file.write_all(b"int otro() { return 0; }").unwrap();
        let index_file_path = format!("{}/index", &git_dir);
        add::add(
            "tests/pull_request/server/merge_conflicts/src/5.c",
            &index_file_path,
            &git_dir,
            "",
            None,
        )
        .unwrap();

        let commit_message = "Fourth commit";
        let commit_3_hash = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let mut repo = Repository::load(repo_name, &root_dir)?;
        let pr = PullRequestCreate {
            title: "title".to_string(),
            description: "description".to_string(),
            source_branch: feature_branch.to_string(),
            target_branch: "main".to_string(),
        };

        PullRequest::new(&mut repo, pr)?;
        repo.dump(&dir)?;

        let repo = Repository::load(repo_name, &dir)?;
        let pr = repo.get_pull_request(1).unwrap();

        let mut repo = Repository::load(repo_name, &dir)?;
        let result = pr.merge(&dir, ".mgit", &mut repo);

        assert!(result.is_ok());
        let merge_commit_hash = result.unwrap();
        let merge_commit = commit::is_merge_commit(&merge_commit_hash, &git_dir).unwrap();
        assert!(merge_commit);

        let merge_commit_parents = commit::get_merge_parents(&merge_commit_hash, &git_dir).unwrap();
        assert_eq!(merge_commit_parents.len(), 2);
        assert!(merge_commit_parents.contains(&commit_2_hash));
        assert!(merge_commit_parents.contains(&commit_3_hash));

        let main_commit_hash = branch::get_branch_commit_hash("main", &git_dir).unwrap();
        assert_eq!(main_commit_hash, merge_commit_hash);

        let repo_path = format!("tests/pull_request/server/merge_conflicts");
        std::fs::remove_dir_all(repo_path)?;

        Ok(())
    }
}
