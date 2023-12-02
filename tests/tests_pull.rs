use messi::{pull::{self, git_pull}, init::git_init, remote::git_remote, git_config::git_config, config::Config};
use std::{io, fs, path::Path};

#[test]
#[ignore]
fn remove_dir() -> io::Result<()> {
    let git_dir = "tests/pull/";
    fs::remove_dir_all(git_dir)?;
    Ok(())
}

#[test]
#[ignore]
fn test_pull() -> io::Result<()> {
    let git_dir = "tests/pull";
    let result = run_test(git_dir);
    // fs::remove_dir_all(git_dir)?;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

fn run_test(git_dir: &str) -> io::Result<()> {
    git_init(git_dir, "master", None)?;
    let line = vec!["add", "origin", "localhost:9418/repo"];
    let repo_path = git_dir.to_owned() + "/.mgit";
    let mut config = Config::load(&repo_path)?;
    git_remote(&mut config, line, &mut vec![])?;
    git_pull("master", git_dir, None, "localhost")?;
    
    let files_count = fs::read_dir(repo_path)?.count();
    if files_count < 7 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error: Files not copied\n",
        ));
    }
    Ok(())
}