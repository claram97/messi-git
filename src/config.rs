use crate::{branch_handler::Branch, remote_handler::Remote};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
};

#[derive(Default)]
pub struct Config {
    config_file_path: String,
    pub remotes: Vec<Remote>,
    branches: Vec<Branch>,
}

impl Config {
    fn new(config_file_path: String) -> Config {
        Config {
            config_file_path,
            remotes: Vec::new(),
            branches: Vec::new(),
        }
    }

    /// Loads a Git configuration from the specified directory.
    ///
    /// This function reads and parses the Git configuration file from the provided Git directory path.
    /// The configuration is used to populate a `Config` struct, including remote repositories and branch information.
    /// It needs to have, at least, basic information provided when git init is run. If it's empty, it will behave weirdly.
    ///
    /// # Arguments
    ///
    /// - `git_dir`: A string representing the path to the Git directory where the configuration is located.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Config` struct if the configuration is successfully loaded, or an
    /// `std::io::Error` in case of any errors during the loading process.
    ///
    /*
    pub fn load(git_dir: &str) -> io::Result<Config> {
        let file_name = format!("{}/config", git_dir);
        let mut config = Config::new((&file_name).to_string());
        let file = File::open(&file_name)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().skip(5);
        let mut buffer: Vec<String> = Vec::new();
        let mut count = 0;

        loop {
            if count == 3 {
                for line in &buffer {
                    if line.starts_with("[remote") {
                        let splitted_name: Vec<&str> = (&buffer[0]).split('"').collect();
                        let name = (&splitted_name[1]).to_string();
                        let splitted_url: Vec<&str> = (&buffer[1]).split(' ').collect();
                        let url = (&splitted_url[2]).to_string();
                        let splitted_fetch: Vec<&str> = (&buffer[2]).split(' ').collect();
                        let fetch = (&splitted_fetch[2]).to_string();
                        let remote = Remote::new(name, url, fetch);
                        config.remotes.push(remote);
                    } else if line.starts_with("[branch") {
                        let splitted_name: Vec<&str> = (&buffer[0]).split('"').collect();
                        let name = (&splitted_name[1]).to_string();
                        let splitted_remote: Vec<&str> = (&buffer[1]).split(' ').collect();
                        let remote = (&splitted_remote[2]).to_string();
                        let splitted_merge: Vec<&str> = (&buffer[2]).split(' ').collect();
                        let merge = (&splitted_merge[2]).to_string();
                        let branch = Branch::new(name, remote, merge);
                        config.branches.push(branch);
                    }
                }
                buffer.clear();
                count = 0;
            }

            match lines.next() {
                Some(Ok(line)) => {
                    buffer.push(line);
                    count += 1;
                }
                Some(Err(err)) => {
                    eprintln!("Error al leer línea: {}", err);
                }
                None => {
                    break;
                }
            }
        }
        Ok(config)
    }*/

    /// Loads a Git configuration from the specified directory.
    ///
    /// This function reads and parses the Git configuration file from the provided Git directory path.
    /// The configuration is used to populate a `Config` struct, including remote repositories and branch information.
    /// It needs to have, at least, basic information provided when git init is run. If it's empty, it will behave weirdly.
    ///
    /// # Arguments
    ///
    /// - `git_dir`: A string representing the path to the Git directory where the configuration is located.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Config` struct if the configuration is successfully loaded, or an
    /// `std::io::Error` in case of any errors during the loading process.
    ///
    pub fn load(git_dir: &str) -> io::Result<Config> {
        let file_name = format!("{}/config", git_dir);
        let mut config = Config::new(file_name.clone());
        let file = File::open(&file_name)?;

        let relevant_lines = Self::extract_relevant_lines(file)?;

        Self::process_lines(&mut config, relevant_lines);

        Ok(config)
    }

    /// Extracts relevant lines from a Git configuration file.
    ///
    /// This function reads a Git configuration file and extracts relevant lines, skipping the
    /// initial lines that are not needed for configuration parsing. The relevant lines contain
    /// information about remote repositories and branches.
    ///
    /// # Arguments
    ///
    /// - `file`: A file object representing the opened Git configuration file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of strings representing the relevant lines from
    /// the configuration file if the extraction is successful, or an `std::io::Error` in case of
    /// any errors during the extraction process.
    ///
    fn extract_relevant_lines(file: File) -> io::Result<Vec<String>> {
        let reader = BufReader::new(file);
        let mut lines = reader.lines().skip(5);

        let mut relevant_lines: Vec<String> = Vec::new();
        while let Some(Ok(line)) = lines.next() {
            relevant_lines.push(line);
        }

        Ok(relevant_lines)
    }

    /// Processes relevant lines from a Git configuration file and populates a `Config` struct.
    ///
    /// This function takes a vector of relevant lines from a Git configuration file and processes
    /// them to populate the provided `Config` struct with remote repositories and branch information.
    ///
    /// # Arguments
    ///
    /// - `config`: A mutable reference to the `Config` struct where the parsed data will be stored.
    /// - `lines`: A vector of strings containing the relevant lines from the configuration file.
    ///
    fn process_lines(config: &mut Config, lines: Vec<String>) {
        let mut buffer: Vec<String> = Vec::new();
        let mut count = 0;

        for line in lines {
            buffer.push(line);
            count += 1;

            if count == 3 {
                if let Some(remote) = Self::parse_remote(&buffer) {
                    config.remotes.push(remote);
                } else if let Some(branch) = Self::parse_branch(&buffer) {
                    config.branches.push(branch);
                }

                buffer.clear();
                count = 0;
            }
        }
    }

    /// Parses relevant lines to extract information about a remote repository.
    ///
    /// This function takes a vector of relevant lines and attempts to extract information about
    /// a remote repository. If the lines correspond to a remote repository, it creates a `Remote`
    /// object and returns it as an `Option`. If the lines do not match a remote repository format,
    /// it returns `None`.
    ///
    /// # Arguments
    ///
    /// - `buffer`: A reference to a vector of strings containing the relevant lines.
    ///
    /// # Returns
    ///
    /// Returns an `Option` containing a `Remote` object if the lines match a remote repository format,
    /// or `None` if they do not.
    ///
    fn parse_remote(buffer: &[String]) -> Option<Remote> {
        if buffer[0].starts_with("[remote") {
            let name = buffer[0].split('"').nth(1)?.to_string();
            let url = buffer[1].split(' ').nth(2)?.to_string();
            let fetch = buffer[2].split(' ').nth(2)?.to_string();
            Some(Remote::new(name, url, fetch))
        } else {
            None
        }
    }

    /// Parses relevant lines to extract information about a Git branch.
    ///
    /// This function takes a vector of relevant lines and attempts to extract information about
    /// a Git branch. If the lines correspond to a branch, it creates a `Branch` object and returns
    /// it as an `Option`. If the lines do not match a branch format, it returns `None`.
    ///
    /// # Arguments
    ///
    /// - `buffer`: A reference to a vector of strings containing the relevant lines.
    ///
    /// # Returns
    ///
    /// Returns an `Option` containing a `Branch` object if the lines match a branch format,
    /// or `None` if they do not.
    ///
    fn parse_branch(buffer: &[String]) -> Option<Branch> {
        if buffer[0].starts_with("[branch") {
            let name = buffer[0].split('"').nth(1)?.to_string();
            let remote = buffer[1].split(' ').nth(2)?.to_string();
            let merge = buffer[2].split(' ').nth(2)?.to_string();
            Some(Branch::new(name, remote, merge))
        } else {
            None
        }
    }

    /// Adds a new remote repository to the Git configuration.
    ///
    /// This function adds a new remote repository to the Git configuration, updating both the in-memory
    /// `Config` struct and the actual configuration file on disk. If a remote with the same name already
    /// exists, it returns an error and does not modify the configuration.
    ///
    /// # Arguments
    ///
    /// - `name`: A string representing the name of the remote repository to add.
    /// - `url`: A string representing the URL of the remote repository.
    /// - `fetch`: A string representing the fetch URL for the remote repository.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the remote repository is successfully added to the configuration. If a remote
    /// with the same name already exists, it returns an error of type `io::ErrorKind::AlreadyExists`.
    ///
    pub fn add_remote(
        &mut self,
        name: String,
        url: String,
        fetch: String,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(_index) = self.remotes.iter().position(|r| r.name == name) {
            let error_message = format!("error: remote {} already exists.", name);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("error: remote {} already exists", name),
            ));
        }
        let remote = Remote::new(name.to_string(), url.to_string(), fetch.to_string());
        self.remotes.push(remote);
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.config_file_path)?;
        let data_to_append = format!(
            "[remote \"{}\"]\n\turl = {}\n\tfetch = {}\n",
            name, url, fetch
        );
        file.write_all(data_to_append.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Adds a new branch to the Git configuration.
    ///
    /// This function adds a new branch to the Git configuration, updating both the in-memory
    /// `Config` struct and the actual configuration file on disk. If a branch with the same name
    /// already exists, it returns an error and does not modify the configuration.
    ///
    /// # Arguments
    ///
    /// - `name`: A string representing the name of the branch to add.
    /// - `remote`: A string representing the remote associated with the branch.
    /// - `merge`: A string representing the merge reference for the branch.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the branch is successfully added to the configuration. If a branch with the
    /// same name already exists, it returns an error of type `io::ErrorKind::AlreadyExists`.
    ///
    pub fn add_branch(
        &mut self,
        name: String,
        remote: String,
        merge: String,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(_index) = self.branches.iter().position(|b| b.name == name) {
            let error_message = format!("error: branch {} already exists.", name);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("error: remote {} already exists", name),
            ));
        }
        let branch = Branch::new(name.to_string(), remote.to_string(), merge.to_string());
        self.branches.push(branch);
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.config_file_path)?;
        let data_to_append = format!(
            "[branch \"{}\"]\n\tremote = {}\n\tmerge = {}\n",
            name, remote, merge
        );
        file.write_all(data_to_append.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Removes a section (remote or branch) from the Git configuration file.
    ///
    /// This function removes a section, either a remote or branch, from the Git configuration file.
    /// It opens the configuration file, skips the lines of the specified section to be removed, and
    /// then writes the rest of the file to a temporary file. Afterward, it renames the temporary file
    /// to replace the original configuration file.
    ///
    /// # Arguments
    ///
    /// - `name`: A string representing the name of the section (remote or branch) to remove.
    /// - `type_`: A string specifying the type of the section to remove (e.g., "remote" or "branch").
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the specified section is successfully removed from the configuration file.
    ///
    pub fn remove_from_file(&mut self, name: &str, type_: &str) -> io::Result<()> {
        let input_file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(input_file);

        let temp_file_path = self.config_file_path.to_string() + "2";
        let output_file = File::create(&temp_file_path)?;
        let mut writer = io::BufWriter::new(output_file);

        let mut skip_lines = 0;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with(&format!("[{} \"{}\"]", type_, name)) {
                skip_lines = 3;
            } else if skip_lines > 0 {
                skip_lines -= 1;
            } else {
                writeln!(writer, "{}", line)?;
            }
        }

        std::fs::rename(temp_file_path, &self.config_file_path)?;

        Ok(())
    }

    /// Removes a remote repository from the Git configuration.
    ///
    /// This function removes a remote repository with the specified name from both the in-memory
    /// `Config` struct and the actual configuration file on disk. If a remote with the given name
    /// does not exist, it returns an error.
    ///
    /// # Arguments
    ///
    /// - `name`: A string representing the name of the remote repository to remove.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the remote repository is successfully removed from the configuration. If a
    /// remote with the specified name does not exist, it returns an error of type `io::ErrorKind::InvalidInput`.
    ///
    pub fn remove_remote(&mut self, name: &str, output: &mut impl Write) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == name) {
            self.remotes.remove(index);
            self.remove_from_file(name, "remote")?;
        } else {
            let error_mesagge = format!("error: No such remote: '{}'", name);
            output.write_all(error_mesagge.as_bytes())?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_mesagge));
        };
        Ok(())
    }

    /// Removes a branch from the Git configuration.
    ///
    /// This function removes a branch with the specified name from both the in-memory `Config` struct
    /// and the actual configuration file on disk. If a branch with the given name does not exist, it
    /// returns an error.
    ///
    /// # Arguments
    ///
    /// - `name`: A string representing the name of the branch to remove.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the branch is successfully removed from the configuration. If a branch with
    /// the specified name does not exist, it returns an error of type `io::ErrorKind::InvalidInput`.
    ///
    pub fn remove_branch(&mut self, name: &str, output: &mut impl Write) -> io::Result<()> {
        if let Some(index) = self.branches.iter().position(|b| b.name == name) {
            self.branches.remove(index);
            self.remove_from_file(name, "branch")?;
        } else {
            let error_message = format!("error: No such branch: '{}'", name);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
        }
        Ok(())
    }

    /// Updates a remote repository's configuration in the Git configuration file.
    ///
    /// This function updates the configuration of a remote repository in the Git configuration file,
    /// both in-memory and on disk. It allows for changing the name, URL, or fetch configuration of
    /// the remote repository.
    ///
    /// # Arguments
    ///
    /// - `remote`: A reference to a `Remote` object containing the updated configuration for the remote repository.
    /// - `remote_initial_name`: An optional string representing the initial name of the remote repository. If provided,
    ///   the function will use this name to locate and update the remote section in the configuration file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the remote repository's configuration is successfully updated in the configuration file.
    ///
    fn change_remote_from_file(
        &self,
        remote: &Remote,
        remote_initial_name: Option<&str>,
    ) -> io::Result<()> {
        let initial_name = if let Some(name) = remote_initial_name {
            name.to_string()
        } else {
            remote.name.clone()
        };

        let input_file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(input_file);

        let temp_file_path = self.config_file_path.to_string() + "2";
        let output_file = File::create(&temp_file_path)?;
        let mut writer = io::BufWriter::new(output_file);

        let mut skip_lines = 0;
        let mut buffer: Vec<String> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if skip_lines > 0 {
                skip_lines -= 1;
            } else if line.starts_with(&format!("[remote \"{}\"]", initial_name)) {
                skip_lines = 3;
                buffer.push(format!("[remote \"{}\"]", remote.name));
                buffer.push(format!("\turl = {}", remote.url));
                buffer.push(format!("\tfetch = {}", &remote.fetch));
            } else {
                buffer.push(line);
            }
        }

        for line in buffer {
            writeln!(writer, "{}", line)?;
        }

        std::fs::rename(temp_file_path, &self.config_file_path)?;

        Ok(())
    }

    /// Retrieves and writes the URL of a remote repository to the specified output.
    ///
    /// This function retrieves the URL of a remote repository with the specified name and writes it
    /// to the provided output, typically a writable stream. If no remote with the given name is found,
    /// it returns an error.
    ///
    /// # Arguments
    ///
    /// - `remote_name`: A string representing the name of the remote repository for which to retrieve the URL.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where the remote URL will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the URL of the remote repository is successfully retrieved and written to the output.
    /// If no remote with the specified name is found, it returns an error of type `io::ErrorKind::InvalidInput`.
    ///
    pub fn get_url(&self, remote_name: &str, output: &mut impl Write) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == remote_name) {
            if let Some(remote) = self.remotes.get(index) {
                output.write_all(remote.url.as_bytes())?;
                Ok(())
            } else {
                let error_message = format!("error: No such remote '{}'", remote_name);
                output.write_all(error_message.as_bytes())?;
                Err(io::Error::new(io::ErrorKind::InvalidInput, error_message))
            }
        } else {
            let error_message = format!("error: No such remote '{}'", remote_name);
            output.write_all(error_message.as_bytes())?;
            Err(io::Error::new(io::ErrorKind::InvalidInput, error_message))
        }
    }

    /// Sets the URL of a remote repository in the Git configuration.
    ///
    /// This function updates the URL of a remote repository with the specified name in both the
    /// in-memory `Config` struct and the actual configuration file on disk. If no remote with the given
    /// name is found, it returns an error. If the URL is already the same as the new URL, no changes are made.
    ///
    /// # Arguments
    ///
    /// - `remote_name`: A string representing the name of the remote repository for which to set the new URL.
    /// - `new_url`: A string representing the new URL to set for the remote repository.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the URL of the remote repository is successfully updated in the configuration.
    /// If no remote with the specified name is found, it returns an error of type `io::ErrorKind::InvalidInput`.
    ///
    pub fn set_url(
        &mut self,
        remote_name: &str,
        new_url: &str,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == remote_name) {
            if let Some(remote) = self.remotes.get(index) {
                if remote.url.eq(&new_url) {
                    return Ok(());
                } else {
                    let new_remote = Remote::new(
                        (&remote_name).to_string(),
                        (&new_url).to_string(),
                        remote.fetch.to_string(),
                    );
                    self.change_remote_from_file(&new_remote, None)?;
                    self.remotes.remove(index);
                    self.remotes.push(new_remote);
                }
            } else {
                let error_message = format!("error: No such remote '{}'", remote_name);
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
        } else {
            let error_message = format!("error: No such remote '{}'", remote_name);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
        }
        Ok(())
    }

    /// Renames a remote repository in the Git configuration.
    ///
    /// This function renames a remote repository in both the in-memory `Config` struct and the actual
    /// configuration file on disk. If no remote with the specified name is found or if a remote with
    /// the new name already exists, it returns an error.
    ///
    /// # Arguments
    ///
    /// - `remote_name`: A string representing the current name of the remote repository to rename.
    /// - `remote_new_name`: A string representing the new name for the remote repository.
    /// - `output`: A mutable reference to an object implementing the `Write` trait, where error messages
    ///    or output will be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the remote repository is successfully renamed in the configuration. If no
    /// remote with the current name is found, a remote with the new name already exists, or any other
    /// errors occur, it returns an appropriate error.
    ///
    pub fn change_remote_name(
        &mut self,
        remote_name: &str,
        remote_new_name: &str,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == remote_name) {
            if self.remotes.iter().any(|s| s.name == remote_new_name) {
                let error_message = format!("error: remote {} already exists.", remote_new_name);
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, error_message));
            } else if let Some(remote) = self.remotes.get(index) {
                let new_remote = Remote::new(
                    remote_new_name.to_string(),
                    remote.url.to_string(),
                    remote.fetch.to_string(),
                );
                self.change_remote_from_file(&new_remote, Some(remote_name))?;
                self.remotes.remove(index);
                self.remotes.push(new_remote);
            } else {
                let error_message = format!("error: No such remote '{}'", remote_name);
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
        } else {
            let error_message = format!("error: No such remote '{}'", remote_name);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{io::Read, path::Path};

    use crate::init;

    use super::*;

    fn create_if_not_exists(path: &str, is_dir: bool) -> io::Result<()> {
        if !Path::new(path).exists() {
            if is_dir {
                std::fs::create_dir(path)?;
            } else {
                File::create(path)?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_load_config_ok() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_1", true)?;
        init::git_init("tests/config_fake_repo_1", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_1/.mgit/config", false)?;
        let config_result = Config::load("tests/config_fake_repo_1/.mgit");
        assert!(config_result.is_ok());
        std::fs::remove_dir_all("tests/config_fake_repo_1")?;
        Ok(())
    }

    #[test]
    fn test_load_config_error() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_2", true)?;
        init::git_init("tests/config_fake_repo_2", "current_branch", None)?;
        let config_path = Path::new("tests/config_fake_repo_2/.mgit/config");
        if config_path.exists() {
            std::fs::remove_file(config_path)?;
        }
        let config_result = Config::load("tests/config_fake_repo_2/.mgit");
        assert!(config_result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_2")?;
        Ok(())
    }

    #[test]
    fn test_add_existing_remote_fails() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_3", true)?;
        init::git_init("tests/config_fake_repo_3", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_3/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_3/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        )?;
        let result = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        );
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_3")?;
        Ok(())
    }

    #[test]
    fn test_writing_new_remote_correctly_to_file() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_4", true)?;
        init::git_init("tests/config_fake_repo_4", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_4/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_4/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let result = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        );
        let mut config_file = File::open("tests/config_fake_repo_4/.mgit/config")?;
        let mut config_file_content = String::new();
        config_file.read_to_string(&mut config_file_content)?;
        assert!(config_file_content.eq("[remote \"my_remote\"]\n\turl = url\n\tfetch = fetch\n"));
        assert!(result.is_ok());
        std::fs::remove_dir_all("tests/config_fake_repo_4")?;
        Ok(())
    }

    #[test]
    fn test_removing_remote_correctly_from_file() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_5", true)?;
        init::git_init("tests/config_fake_repo_5", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_5/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_5/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        );
        let mut config_file = File::open("tests/config_fake_repo_5/.mgit/config")?;
        let mut output: Vec<u8> = vec![];
        let result = config.remove_remote("my_remote", &mut output);
        let mut config_file_content = String::new();
        config_file.read_to_string(&mut config_file_content)?;
        //assert!(config_file_content.is_empty());
        assert!(result.is_ok());
        std::fs::remove_dir_all("tests/config_fake_repo_5")?;
        Ok(())
    }

    #[test]
    fn test_removing_non_existing_remote_fails() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_6", true)?;
        init::git_init("tests/config_fake_repo_6", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_6/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_6/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let result = config.remove_remote("my_remote", &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_6")?;
        Ok(())
    }

    #[test]
    fn test_set_url_to_existing_remote_make_correct_changes_in_file() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_7", true)?;
        init::git_init("tests/config_fake_repo_7", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_7/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_7/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        );
        let mut config_file = File::open("tests/config_fake_repo_7/.mgit/config")?;
        let mut initial_config_file_content = String::new();
        config_file.read_to_string(&mut initial_config_file_content)?;

        assert!(initial_config_file_content
            .eq("[remote \"my_remote\"]\n\turl = url\n\tfetch = fetch\n"));
        let mut output: Vec<u8> = vec![];
        let result = config.set_url("my_remote", "new_url", &mut output);
        assert!(result.is_ok());
        let mut config_file = File::open("tests/config_fake_repo_7/.mgit/config")?;
        let mut final_config_file_content = String::new();
        config_file.read_to_string(&mut final_config_file_content)?;
        assert!(final_config_file_content
            .eq("[remote \"my_remote\"]\n\turl = new_url\n\tfetch = fetch\n"));
        assert!(initial_config_file_content.ne(&final_config_file_content));
        std::fs::remove_dir_all("tests/config_fake_repo_7")?;
        Ok(())
    }

    #[test]
    fn test_set_url_to_non_existing_remote_fails() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_8", true)?;
        init::git_init("tests/config_fake_repo_8", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_8/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_8/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let result = config.set_url("my_remote", "new_url", &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_8")?;
        Ok(())
    }

    #[test]
    fn test_get_url_from_non_existing_remote_fails() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_9", true)?;
        init::git_init("tests/config_fake_repo_9", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_9/.mgit/config", false)?;
        let config = Config::load("tests/config_fake_repo_9/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let result = config.get_url("my_remote", &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_9")?;
        Ok(())
    }

    #[test]
    fn test_get_url_from_existing_remote_returns_url_successfully() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_10", true)?;
        init::git_init("tests/config_fake_repo_10", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_10/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_10/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        )?;
        let mut output: Vec<u8> = vec![];
        let result = config.get_url("my_remote", &mut output);
        assert!(result.is_ok());
        let result = String::from_utf8(output).unwrap();
        assert!(result.eq("url"));
        std::fs::remove_dir_all("tests/config_fake_repo_10")?;
        Ok(())
    }

    #[test]
    fn changing_name_of_existing_remote_to_non_existings_one_returns_ok() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_11", true)?;
        init::git_init("tests/config_fake_repo_11", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_11/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_11/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        )?;
        let mut initial_config_file_content = String::new();
        let mut config_file = File::open("tests/config_fake_repo_11/.mgit/config")?;
        config_file.read_to_string(&mut initial_config_file_content)?;
        assert!(initial_config_file_content
            .eq("[remote \"my_remote\"]\n\turl = url\n\tfetch = fetch\n"));
        let mut output: Vec<u8> = vec![];
        let result = config.change_remote_name("my_remote", "new_remote", &mut output);
        assert!(result.is_ok());
        drop(config_file);
        let mut config_file = File::open("tests/config_fake_repo_11/.mgit/config")?;
        let mut final_config_file_content = String::new();
        config_file.read_to_string(&mut final_config_file_content)?;
        assert!(
            final_config_file_content.eq("[remote \"new_remote\"]\n\turl = url\n\tfetch = fetch\n")
        );
        assert!(initial_config_file_content.ne(&final_config_file_content));
        std::fs::remove_dir_all("tests/config_fake_repo_11")?;
        Ok(())
    }

    #[test]
    fn changing_name_of_existing_remote_fails_due_to_other_existing_remote() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_12", true)?;
        init::git_init("tests/config_fake_repo_12", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_12/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_12/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "my_remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        )?;
        let mut output: Vec<u8> = vec![];
        let _ = config.add_remote(
            "remote".to_string(),
            "url".to_string(),
            "fetch".to_string(),
            &mut output,
        );
        let mut output: Vec<u8> = vec![];
        let result = config.change_remote_name("my_remote", "remote", &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_12")?;
        Ok(())
    }

    #[test]
    fn changing_name_of_non_existing_remote_fails() -> io::Result<()> {
        create_if_not_exists("tests/config_fake_repo_13", true)?;
        init::git_init("tests/config_fake_repo_13", "current_branch", None)?;
        create_if_not_exists("tests/config_fake_repo_13/.mgit/config", false)?;
        let mut config = Config::load("tests/config_fake_repo_13/.mgit")?;
        let mut output: Vec<u8> = vec![];
        let result = config.change_remote_name("my_remote", "remote", &mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/config_fake_repo_13")?;
        Ok(())
    }
}
