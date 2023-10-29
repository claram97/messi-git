use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

use crate::{commit, utils};

/// Returns the path inside the HEAD file.
/// The one that contains the path to the current branch.
/// If the file is empty, it returns an error.
fn get_current_branch_path(git_dir_path: &str) -> io::Result<String> {
    let head_path = git_dir_path.to_string() + "/HEAD";
    let mut head_file = std::fs::File::open(head_path)?;
    let mut head_content = String::new();
    head_file.read_to_string(&mut head_content)?;
    let path = match head_content.split(' ').last() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HEAD file is empty\n",
            ))
        }
    };
    let nombre: Vec<&str> = path.split('\n').collect();
    let path_final = nombre[0];
    Ok(path_final.to_string())
}

/// Creates a new branch in the repo with the given name.
/// The new branch will point to the same commit as the current branch.
/// HEAD won't be updated.
///
/// ## Arguments
/// * `git_dir` - The path to the repo directory.
/// * `branch_name` - The name of the new branch.
/// * `output` - The output to write the error message if any.
///
/// ## Errors
/// If the branch already exists, the branch is not created and an error is returned.
/// If the HEAD file is empty, an error is returned.
/// If there are no tracked files, an error is returned.
pub fn create_new_branch(
    git_dir: &str,
    branch_name: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(heads_dir)?;
    if entries.count() == 0 {
        let buffer = "fatal: Please commit something to create a branch\n".to_string();
        output.write_all(buffer.as_bytes())?;
        return Ok(());
    }

    let new_refs = (&git_dir).to_string() + "/refs/heads/" + branch_name;
    let refs_path = Path::new(&new_refs);
    if refs_path.exists() {
        let buffer = format!("fatal: A branch named '{}' already exists\n", branch_name);
        output.write_all(buffer.as_bytes())?;
        return Ok(());
    }
    let branch_path = get_current_branch_path(git_dir)?;
    let complete_path = (&git_dir).to_string() + "/" + &branch_path;
    let mut current_branch_file = File::open(complete_path)?;
    let mut content = String::new();
    current_branch_file.read_to_string(&mut content)?;

    let mut file = File::create(&new_refs)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
pub fn list_branches_string(git_dir: &str) -> io::Result<Vec<String>> {
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(&heads_dir)?;
    let current_branch = commit::get_branch_name(git_dir)?;
    let mut branches_output = Vec::new();

    if entries.count() > 0 {
        let entries = fs::read_dir(&heads_dir)?;
        for entry in entries {
            let entry = entry?;
            if current_branch.eq(&entry.file_name().to_string_lossy().to_string()) {
                let branch_line = format!("*\x1B[32m {}\x1B[0m", entry.file_name().to_string_lossy());
                branches_output.push(branch_line);
            } else {
                let branch_line = format!("  {}", entry.file_name().to_string_lossy());
                branches_output.push(branch_line);
            }
        }
    }

    Ok(branches_output)
}


/// Lists all the branches in the repo. It writes the output in the given output.
/// If the branch is the current one, it will be marked with a `*` and in green.
pub fn list_branches(git_dir: &str, output: &mut impl Write) -> io::Result<()> {
    let heads_dir = (&git_dir).to_string() + "/refs/heads";
    let entries = fs::read_dir(&heads_dir)?;
    let current_branch = commit::get_branch_name(git_dir)?;
    if entries.count() > 0 {
        let entries = fs::read_dir(&heads_dir)?;
        for entry in entries {
            let entry = entry?;
            if current_branch.eq(&entry.file_name().to_string_lossy().to_string()) {
                let buffer = format!("*\x1B[32m {}\x1B[0m\n", entry.file_name().to_string_lossy());
                output.write_all(buffer.as_bytes())?;
            } else {
                let buffer = format!("  {}\n", entry.file_name().to_string_lossy());
                output.write_all(buffer.as_bytes())?;
            }
        }
    }

    Ok(())
}

/// Lists all the branches in the repo or creates a new branch depending on the argument.
///
/// ## Arguments
/// * `name` - The name of the new branch. If it's `None`, the current branches are listed.
///
/// ## Errors
/// If the branch already exists, the branch is not created and an error is returned.
/// If the HEAD file is empty, an error is returned.
/// If there are no tracked files, an error is returned.
/// If the git directory is not found, an error is returned.
pub fn git_branch(name: Option<String>) -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match utils::find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ))
        }
    };

    if name.is_some() {
        create_new_branch(&git_dir, &name.unwrap(), &mut io::stdout())?;
    } else {
        list_branches(&git_dir, &mut io::stdout())?;
    }
    Ok(())
}
pub fn git_branch2(name: Option<String>) -> io::Result<String> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match utils::find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ))
        }
    };

    if let Some(branch_name) = name {
        create_new_branch(&git_dir, &branch_name, &mut io::stdout())?;
        Ok("Branch created successfully".to_string())
    } else {
        let mut output: Vec<u8> = vec![];
        list_branches(&git_dir, &mut output)?;
        
        // Convierte los bytes en una cadena y la devuelve
        if let Ok(output_string) = String::from_utf8(output) {
            Ok(output_string)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to convert branch list to string\n",
            ))
        }
    }
}


#[cfg(test)]
mod tests {
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
    fn test_list_branches() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches", true)?;
        init::git_init("tests/test_list_branches", "current_branch", None)?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_1", false)?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_2", false)?;
        create_if_not_exists(
            "tests/test_list_branches/.mgit/refs/heads/current_branch",
            false,
        )?;
        create_if_not_exists("tests/test_list_branches/.mgit/refs/heads/branch_3", false)?;
        let mut output: Vec<u8> = vec![];
        list_branches("tests/test_list_branches/.mgit", &mut output)?;
        assert!(!output.is_empty());
        std::fs::remove_dir_all("tests/test_list_branches")?;
        Ok(())
    }

    #[test]
    fn test_list_branches_empty() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_2", true)?;
        init::git_init("tests/test_list_branches_2", "current_branch", None)?;
        let mut output: Vec<u8> = vec![];
        list_branches("tests/test_list_branches_2/.mgit", &mut output)?;
        assert!(output.is_empty());
        std::fs::remove_dir_all("tests/test_list_branches_2")?;
        Ok(())
    }

    #[test]
    fn test_create_new_branch_already_exists() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_3", true)?;
        init::git_init("tests/test_list_branches_3", "current_branch", None)?;
        create_if_not_exists(
            "tests/test_list_branches_3/.mgit/refs/heads/current_branch",
            false,
        )?;
        let mut output: Vec<u8> = vec![];
        create_new_branch(
            "tests/test_list_branches_3/.mgit",
            "current_branch",
            &mut output,
        )?;
        assert!(!output.is_empty());
        let result = String::from_utf8(output);
        if result.is_ok() {
            let string = result.unwrap();
            assert!(string.starts_with("fatal: A branch named 'current_branch' already exists\n"));
            //Acá mirar directamente si el mensaje es el esperado
            //Acá mirar directamente si el mensaje es el esperado
        }

        std::fs::remove_dir_all("tests/test_list_branches_3")?;
        Ok(())
    }

    #[test]
    fn test_create_new_branch() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_4", true)?;
        init::git_init("tests/test_list_branches_4", "current_branch", None)?;
        create_if_not_exists(
            "tests/test_list_branches_4/.mgit/refs/heads/current_branch",
            false,
        )?;

        let mut current_branch_file =
            File::create("tests/test_list_branches_4/.mgit/refs/heads/current_branch")?;
        let commit_hash = "aaaaaaaaaaaaaaaaaaaaaa";
        current_branch_file.write_all(commit_hash.as_bytes())?;

        let mut output: Vec<u8> = vec![];
        create_new_branch("tests/test_list_branches_4/.mgit", "my_branch", &mut output)?;

        let mut head_file = std::fs::File::open("tests/test_list_branches_4/.mgit/HEAD")?;
        let mut head_content = String::new();
        head_file.read_to_string(&mut head_content)?;

        let mut new_branch_file =
            std::fs::File::open("tests/test_list_branches_4/.mgit/refs/heads/my_branch")?;
        let mut new_branch_content = String::new();
        new_branch_file.read_to_string(&mut new_branch_content)?;

        assert_eq!(output.len(), 0); //No output means ok.
        assert_eq!(head_content, "ref: refs/heads/current_branch\n");
        assert_eq!(new_branch_content, commit_hash);

        std::fs::remove_dir_all("tests/test_list_branches_4")?;
        Ok(())
    }
    #[test]
    fn test_create_new_branch_with_no_tracked_files() -> Result<(), io::Error> {
        create_if_not_exists("tests/test_list_branches_5", true)?;
        init::git_init("tests/test_list_branches_5", "current_branch", None)?;
        let mut output: Vec<u8> = vec![];
        create_new_branch("tests/test_list_branches_5/.mgit", "my_branch", &mut output)?;
        assert!(!output.is_empty());
        let result = String::from_utf8(output);
        if result.is_ok() {
            let string = result.unwrap();
            assert!(string.starts_with("fatal: Please commit something to create a branch\n"));
            //Acá mirar directamente si el mensaje es el esperado
        }
        std::fs::remove_dir_all("tests/test_list_branches_5")?;
        Ok(())
    }
}
