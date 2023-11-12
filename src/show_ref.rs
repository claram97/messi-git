use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use std::fs::File;
use std::io::prelude::*;

pub fn git_show_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 2 {
        show_ref(git_dir, output)?;
    } else if line.len() == 3 {
        show_ref_with_options(git_dir, line, output)?;
    } else if line.len() >= 3 {
        if line[2].eq("--verify") {
            verify_ref(git_dir, line, output)?;
        } else if line[2].starts_with("--") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid option specified",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    Ok(())
}

fn verify_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    for i in 3..line.len() {
        let path_to_verify = format!("{}/{}", git_dir, &line[i]);
        let path = Path::new(&path_to_verify);

        if !path.exists() {
            writeln!(output, "fatal: '{}' - not a valid ref\n", &line[i])?;
        } else {
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            writeln!(output, "{}\t{}\n", contents.trim(), &line[i])?;
        }
    }
    Ok(())
}

fn show_ref(git_dir: &str, output: &mut impl Write) -> io::Result<()> {
    let heads_path = format!("{}/{}", git_dir, "refs/heads");
    let tags_path = format!("{}/{}", git_dir, "refs/tags");
    process_files_in_directory(&heads_path, "heads", false, output)?;
    process_files_in_directory(&tags_path, "tags", false, output)?;
    Ok(())
}

fn show_ref_with_options(
    git_dir: &str,
    line: Vec<String>,
    output: &mut impl Write,
) -> io::Result<()> {
    if line[2].eq("--heads") {
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        process_files_in_directory(&heads_path, "heads", false, output)?;
    } else if line[2].eq("--tags") {
        let tags_path: String = format!("{}/{}", git_dir, "refs/tags");
        process_files_in_directory(&tags_path, "tags", false, output)?;
    } else if line[2].eq("--hash") {
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        process_files_in_directory(&heads_path, "heads", true, output)?;
        process_files_in_directory(&tags_path, "tags", true, output)?;
    } else if line[2].eq("--verify") {
        writeln!(output, "fatal: --verify requires a reference")?;
    } else if line[2].starts_with("--") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid option specified",
        ));
    }
    Ok(())
}

fn process_files_in_directory(
    path: &str,
    type_: &str,
    is_hash: bool,
    output: &mut impl Write,
) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            let file_name = match file_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error"));
                }
            };

            let mut file = fs::File::open(&file_path)?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            if is_hash {
                writeln!(output, "{}\n", contents.trim())?;
            } else {
                writeln!(
                    output,
                    "{}\trefs/{}/{}\n",
                    contents.trim(),
                    type_,
                    file_name
                )?;
            }
        }
    }
    Ok(())
}
