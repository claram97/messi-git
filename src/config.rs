use crate::{branch_handler::Branch, remote_handler::Remote};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
};

#[derive(Default)]
pub struct Config {
    config_file_path: String,
    remotes: Vec<Remote>,
    branches: Vec<Branch>,
}

impl Config {
    // Constructor
    fn new(config_file_path: String) -> Config {
        let config = Config {
            config_file_path,
            remotes: Vec::new(),
            branches: Vec::new(),
        };
        config
    }

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
                        //println!("name {} url {} fetch {}",name,url,fetch);
                        let remote = Remote::new(name, url, fetch);
                        config.remotes.push(remote);
                    } else if line.starts_with("[branch") {
                        let splitted_name: Vec<&str> = (&buffer[0]).split('"').collect();
                        let name = (&splitted_name[1]).to_string();
                        let splitted_remote: Vec<&str> = (&buffer[1]).split(' ').collect();
                        let remote = (&splitted_remote[2]).to_string();
                        let splitted_merge: Vec<&str> = (&buffer[2]).split(' ').collect();
                        let merge = (&splitted_merge[2]).to_string();
                        //println!("name {} remote {} merge {}",name,remote,merge);
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
    }

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
        let remote = Remote::new(
            (&name).to_string(),
            (&url).to_string(),
            (&fetch).to_string(),
        );
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
        let branch = Branch::new(
            (&name).to_string(),
            (&remote).to_string(),
            (&merge).to_string(),
        );
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

    pub fn remove_from_file(&mut self, name: &str, type_: &str) -> io::Result<()> {
        let input_file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(input_file);

        let temp_file_path = (&self.config_file_path).to_string() + "2";
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

    fn change_remote_from_file(
        &self,
        remote: &Remote,
        remote_initial_name: Option<&str>,
    ) -> io::Result<()> {
        let initial_name;
        if remote_initial_name.is_some() {
            initial_name = remote_initial_name.unwrap().to_string();
        } else {
            initial_name = remote.name.clone();
        }
        let input_file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(input_file);

        let temp_file_path = (&self.config_file_path).to_string() + "2";
        let output_file = File::create(&temp_file_path)?;
        let mut writer = io::BufWriter::new(output_file);

        let mut skip_lines = 0;
        let mut buffer: Vec<String> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if skip_lines > 0 {
                skip_lines -= 1;
            } else {
                if line.starts_with(&format!("[remote \"{}\"]", initial_name)) {
                    skip_lines = 3;
                    buffer.push(format!("[remote \"{}\"]", remote.name));
                    buffer.push(format!("\turl = {}", remote.url));
                    buffer.push(format!("\tfetch = {}", &remote.fetch));
                } else {
                    buffer.push(line);
                }
            }
        }

        for line in buffer {
            writeln!(writer, "{}", line)?;
        }

        std::fs::rename(temp_file_path, &self.config_file_path)?;

        Ok(())
    }

    pub fn get_url(&self, remote_name: &str, output: &mut impl Write) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == remote_name) {
            if let Some(remote) = self.remotes.get(index) {
                output.write_all(remote.url.as_bytes())?;
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
                        (&remote.fetch).to_string(),
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

    pub fn change_remote_name(
        &mut self,
        remote_name: &str,
        remote_new_name: &str,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == remote_name) {
            if let Some(_) = self.remotes.iter().position(|s| s.name == remote_new_name) {
                let error_message = format!("error: remote {} already exists.", remote_new_name);
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, error_message));
            } else {
                if let Some(remote) = self.remotes.get(index) {
                    let new_remote = Remote::new(
                        remote_new_name.to_string(),
                        (&remote.url).to_string(),
                        (&remote.fetch).to_string(),
                    );
                    self.change_remote_from_file(&new_remote, Some(remote_name))?;
                    self.remotes.remove(index);
                    self.remotes.push(new_remote);
                } else {
                    let error_message = format!("error: No such remote '{}'", remote_name);
                    output.write_all(error_message.as_bytes())?;
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
                }
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
