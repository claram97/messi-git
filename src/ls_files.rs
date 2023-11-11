use std::{io::{self, Write}, fs};

use crate::{index::Index, hash_object};
const BLOB: &str = "blob";

//let current_directory = std::env::current_dir()?;
pub fn git_ls_files(working_dir : &str, git_dir: &str, current_directory : &str, line: Vec<String>, index : &Index, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 1 || (line.len() == 2 && line[1].eq("-c")) {
        for entry in fs::read_dir(current_directory)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_path_str = entry_path.to_string_lossy().to_string();
            if let Ok(relative_entry_path) = entry_path.strip_prefix(working_dir) {
                let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
                if index.contains(&relative_entry_path_str)
                {
                    if entry_path.is_dir() {
                        let cloned_line = line.clone();
                        git_ls_files(working_dir, git_dir, &entry_path_str, cloned_line, index, output)?;
                    }
                    if entry_path.is_file() {
                        let buffer = format!("{}\n", relative_entry_path_str);
                        output.write_all(buffer.as_bytes())?;
                    }
                }
            } else {
                //Devolver error
            }
        }
    }
    else if line.len() == 2 {
        if line[2].eq("-o") {
            for entry in fs::read_dir(current_directory)? {
                let entry = entry?;
                let entry_path = entry.path();
                let entry_path_str = entry_path.to_string_lossy().to_string();
                if let Ok(relative_entry_path) = entry_path.strip_prefix(working_dir) {
                    let relative_entry_path_str = relative_entry_path.to_string_lossy().to_string();
                    if !index.path_should_be_ignored(&relative_entry_path_str)
                        && !index.contains(&relative_entry_path_str)
                    {
                        if entry_path.is_dir() {
                            let cloned_line = line.clone();
                            git_ls_files(working_dir, git_dir, &entry_path_str, cloned_line, index, output)?;
                        }
                        if entry_path.is_file() {
                            let buffer = format!("{}\n", relative_entry_path_str);
                            output.write_all(buffer.as_bytes())?;
                        }
                    }
                } else {
                    //Devolver error
                }
            }
        }
        else if line[2].eq("m") {
            for (path, hash) in index.iter() {
                let complete_path = git_dir.to_string() + "/" + path;
                let new_hash = hash_object::hash_file_content(&complete_path, BLOB)?;
                if hash.ne(&new_hash) {
                    let buffer = format!("{}\n", path);
                    output.write_all(buffer.as_bytes())?;
                }
            }
        }
        else {
            //Return error
        }
    }

    Ok(())
}