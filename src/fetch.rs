use std::io::{self, BufRead, Write};

use crate::{client::Client, config};

pub struct FetchEntry {
    pub commit_hash: String,
    branch_name: String,
    remote_repo_url: String,
}

pub struct FetchHead {
    entries: Vec<FetchEntry>,
}

impl FetchHead {
    pub fn new() -> FetchHead {
        FetchHead {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: FetchEntry) {
        self.entries.push(entry);
    }

    pub fn get_entries(&self) -> &Vec<FetchEntry> {
        &self.entries
    }

    pub fn get_branch_entry(&self, branch_name: &str) -> Option<&FetchEntry> {
        self.entries.iter().find(|&entry| entry.branch_name == branch_name)
    }

    pub fn write_file(&self, path: &str) -> io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for entry in &self.entries {
            let line = format!(
                "{} {} of {}\n",
                entry.commit_hash, entry.branch_name, entry.remote_repo_url
            );
            file.write_all(line.as_bytes())?;
        }
        Ok(())
    }

    pub fn load_file(path: &str) -> io::Result<FetchHead> {
        let file = std::fs::File::open(path)?;
        let mut fetch_head = FetchHead::new();

        for line in io::BufReader::new(file).lines() {
            let line = line?;
            let line_split: Vec<&str> = line.split(' ').collect();
            let commit_hash = line_split[0].to_string();
            let branch_name = line_split[1].to_string();
            let remote_repo_url = line_split[3].to_string();
            let entry = FetchEntry {
                commit_hash,
                branch_name,
                remote_repo_url,
            };
            fetch_head.add_entry(entry);
        }

        Ok(fetch_head)
    }
}

fn get_clean_refs(refs: Vec<String>) -> Vec<String> {
    let clean_refs = refs
        .iter()
        .map(|x| match x.split('/').last() {
            Some(string) => string.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();
    clean_refs
}

// Git fetch places the most recent commit of each branch in the FETCH_HEAD file. It also brings the objects that are not in the local repository.
pub fn git_fetch(remote_repo_name: Option<&str>, host: &str, local_dir: &str) -> io::Result<()> {
    let git_dir = local_dir.to_string() + "/.mgit";
    let config_file = config::Config::load(&git_dir)?;
    let remote_repo_name = remote_repo_name.unwrap_or("origin");

    let remote_repo_url = config_file.get_url(remote_repo_name, &mut io::stdout())?;
    let local_git_dir = local_dir.to_string() + "/.mgit";
    let mut client = Client::new(&remote_repo_url, remote_repo_name, host);
    let refs = client.get_refs()?;
    let clean_refs = get_clean_refs(refs);
    let fetch_head_path = local_git_dir.to_string() + "/FETCH_HEAD";
    let mut fetch_head_file = FetchHead::new();
    for server_ref in clean_refs {
        let result = client.upload_pack(&server_ref, &local_git_dir, "origin");
        match result {
            Ok(commit_hash) => {
                if server_ref != "HEAD" {
                    let entry = FetchEntry {
                        commit_hash,
                        branch_name: server_ref,
                        remote_repo_url: remote_repo_url.clone(),
                    };
                    fetch_head_file.add_entry(entry);
                }
            }
            Err(error) => {
                println!("Error: {:?}", error);
            }
        }
    }

    fetch_head_file.write_file(&fetch_head_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::clone;

    const PORT: &str = "9418";

    #[test]
    fn test_get_clean_refs() {
        let refs = vec![
            "refs/heads/master".to_string(),
            "refs/heads/develop".to_string(),
        ];
        let clean_refs = super::get_clean_refs(refs);

        assert_eq!(clean_refs[0], "master");
        assert_eq!(clean_refs[1], "develop");
    }

    #[ignore = "This test only works if the server is running"]
    #[test]
    fn test_fetch() {
        let local_dir = env::temp_dir().to_str().unwrap().to_string() + "/test_fetch";
        let address = "localhost:".to_owned() + PORT;
        let remote_repo_name = "repo_prueba";
        let host = "localhost";
        let _ = clone::git_clone(&address, remote_repo_name, host, &local_dir);

        let result = super::git_fetch(Some(remote_repo_name), "localhost", &local_dir);
        assert!(result.is_ok());
    }
}
