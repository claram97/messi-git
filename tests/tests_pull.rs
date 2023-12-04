use messi::{config::Config, init::git_init, pull::git_pull, remote::git_remote};
use std::{fs, io, path::Path};

#[test]
#[ignore = "This test only works if the server is running"]
fn test_pull_empty_repo() -> io::Result<()> {
    let git_dir = "tests/pull";
    let result = first_pull(git_dir);
    fs::remove_dir_all(git_dir)?;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[test]
#[ignore = "This test only works if the server is running"]
fn test_pull_non_empty_repo() -> io::Result<()> {
    let git_dir = "tests/pull";
    let result = second_pull(git_dir);
    fs::remove_dir_all(git_dir)?;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn first_pull(git_dir: &str) -> io::Result<()> {
    git_init(git_dir, ".mgit", "master", None)?;
    let line = vec!["add", "origin", "localhost:9418/repo"];
    let repo_path = git_dir.to_owned() + "/.mgit";
    let mut config = Config::load(&repo_path)?;
    git_remote(&mut config, line, &mut vec![])?;
    git_pull("master", git_dir, None, "localhost")?;

    let has_file1 =
        fs::read_dir(git_dir)?.any(|x| x.unwrap().path() == Path::new(git_dir).join("hola.txt"));
    let has_file2 =
        fs::read_dir(git_dir)?.any(|x| x.unwrap().path() == Path::new(git_dir).join("hola2.txt"));

    if !has_file1 || has_file2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Files copied incorrectly",
        ));
    }

    Ok(())
}

fn second_pull(git_dir: &str) -> io::Result<()> {
    first_pull(git_dir)?;
    let line = vec!["remove", "origin"];
    let repo_path = git_dir.to_owned() + "/.mgit";

    let mut config = Config::load(&repo_path)?;
    git_remote(&mut config, line, &mut vec![])?;

    let line = vec!["add", "origin", "localhost:9418/repoff"];
    git_remote(&mut config, line, &mut vec![])?;

    git_pull("master", git_dir, None, "localhost")?;

    let has_file1 =
        fs::read_dir(git_dir)?.any(|x| x.unwrap().path() == Path::new(git_dir).join("hola.txt"));
    let has_file2 =
        fs::read_dir(git_dir)?.any(|x| x.unwrap().path() == Path::new(git_dir).join("hola2.txt"));

    if !has_file1 || !has_file2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "File copied incorrectly",
        ));
    };

    let hola = fs::read(Path::new(git_dir).join("hola.txt"))?;
    let hola2 = fs::read(Path::new(git_dir).join("hola2.txt"))?;

    if hola == hola2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "File copied incorrectly. Must be different",
        ));
    }

    Ok(())
}
