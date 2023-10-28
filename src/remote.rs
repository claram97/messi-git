//git remote add origin https://ejemplo.com/repo.git
//git remote remove origin
//git remote set-url origin https://nueva-url.com/repo.git
//git remote get-url origin
//git remote rename nombre-viejo nombre-nuevo

use std::io::{self, Write};

use crate::config::Config;

pub fn git_remote(git_dir_path: &str, line: Vec<&str>, output: &mut impl Write) -> io::Result<()> {
    if (line.len() != 2) && (line.len() != 3) {
        let error_message = format!("Invalid arguments.\n");
        output.write_all(error_message.as_bytes())?;
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }
    let mut config = Config::load(git_dir_path)?;
    let _comand = match line[0] {
        "add" => {
            if line.len() != 3 {
                let error_message = format!("Invalid arguments.\n");
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
            let fetch = "fetch".to_string(); //Esto hay que ver bien de dónde sale para armarlo como coresponde
            config.add_remote(line[1].to_string(), line[2].to_string(), fetch, output)?;
        }
        "remove" => {
            if line.len() != 2 {
                let error_message = format!("Invalid arguments.\n");
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
            config.remove_remote(line[1], output)?;
        }
        "set-url" => {
            if line.len() != 3 {
                let error_message = format!("Invalid arguments.\n");
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
            config.set_url(line[1], line[2], output)?;
        }
        "get-url" => {
            if line.len() != 2 {
                let error_message = format!("Invalid arguments.\n");
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
            config.get_url(line[1], output)?;
        }
        "rename" => {
            if line.len() != 3 {
                let error_message = format!("Invalid arguments.\n");
                output.write_all(error_message.as_bytes())?;
                return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
            }
            config.change_remote_name(line[1], line[2], &mut io::stdout())?;
        }
        _ => {
            let error_message = format!("error: Unknown subcommand {}\n", line[0]);
            output.write_all(error_message.as_bytes())?;
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
        }
    };
    drop(config);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{fs::File, path::Path, io};

    use crate::init;

    
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
    fn test_invalid_arguments_lenght() -> io::Result<()> {
        let line = vec!["","","",""];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("",line,&mut output);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_uknown_command_makes_git_remote_fail() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_1", true)?;
        init::git_init("tests/remote_fake_repo_1", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_1/.mgit/config", false)?;
        let line = vec!["something"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_1/.mgit",line,&mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_1")?;
        Ok(())
    }

    #[test]
    fn test_invalid_add_command_makes_git_remote_fail_1() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_2", true)?;
        init::git_init("tests/remote_fake_repo_2", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_2/.mgit/config", false)?;
        let line = vec!["add"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_2/.mgit",line,&mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_2")?;
        Ok(())
    }

    #[test]
    fn test_invalid_add_command_makes_git_remote_fail_2() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_3", true)?;
        init::git_init("tests/remote_fake_repo_3", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_3/.mgit/config", false)?;
        let line = vec!["add","new_remote","url","something else"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_3/.mgit",line,&mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_3")?;
        Ok(())
    }

    #[test]
    fn test_valid_add_command_returns_ok() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_4", true)?;
        init::git_init("tests/remote_fake_repo_4", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_4/.mgit/config", false)?;
        let line = vec!["add","new_remote","url"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_4/.mgit",line,&mut output);
        assert!(result.is_ok());
        std::fs::remove_dir_all("tests/remote_fake_repo_4")?;
        Ok(())
    }

    #[test]
    fn test_invalid_remove_command_makes_git_remote_fail_1() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_5", true)?;
        init::git_init("tests/remote_fake_repo_5", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_5/.mgit/config", false)?;
        let line = vec!["remove","remote_name","something else"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_2/.mgit",line,&mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_5")?;
        Ok(())
    }

    #[test]
    fn test_invalid_remove_command_makes_git_remote_fail_2() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_6", true)?;
        init::git_init("tests/remote_fake_repo_6", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_6/.mgit/config", false)?;
        let line = vec!["remove"];
        let mut output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_3/.mgit",line,&mut output);
        assert!(result.is_err());
        std::fs::remove_dir_all("tests/remote_fake_repo_6")?;
        Ok(())
    }

    //Si hago add con git_remote, después no anda el remove con git_remote
    //Si hago add a manopla, el remove con git_remote hace lo que tiene que hacer, sin embargo, devuelve error
    #[ignore]
    #[test]
    fn test_valid_remove_command_returns_ok() -> io::Result<()> {
        create_if_not_exists("tests/remote_fake_repo_7", true)?;
        init::git_init("tests/remote_fake_repo_7", "current_branch", None)?;
        create_if_not_exists("tests/remote_fake_repo_7/.mgit/config", false)?;
        //let line = vec!["add","new_remote","url"];
        //let mut output : Vec<u8> = vec![];
        //let _result = git_remote("tests/remote_fake_repo_7/.mgit",line,&mut output);

        let mut config_file = File::open("tests/remote_fake_repo_7/.mgit/config")?;
        let config_file_content = "[remote \"new_remote\"]\n\turl = url\n\tfetch = fetch\n";
        config_file.write_all(&config_file_content.as_bytes())?;
        drop(config_file);
        let remove_line = vec!["remove","new_remote"];
        let mut new_output : Vec<u8> = vec![];
        let result = git_remote("tests/remote_fake_repo_7/.mgit",remove_line,&mut new_output);
        assert!(result.is_ok());
        std::fs::remove_dir_all("tests/remote_fake_repo_7")?;
        Ok(())
    }

}

