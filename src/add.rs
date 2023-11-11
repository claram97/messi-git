const MGIT: &str = ".mgit";
const OPTIONS_ALL: &str = "-u";

use std::fs;
use std::io;

use crate::ignorer::is_subpath;
use crate::index::Index;

pub fn process_file_name(
    index: &mut Index,
    file_name: &str,
) -> io::Result<()> {
    if fs::metadata(file_name)?.is_dir() {
        // If the file_name is a directory, add all files inside
        for entry in fs::read_dir(file_name)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                index.add_path(file_path.to_str().unwrap())?;
            }
        }
    } else {
        // If the path is a file, add only that file
        index.add_path(file_name)?;
    }

    Ok(())
}

/// Add files to the Git index.
///
/// This function adds files to the Git index based on the provided path.
/// If the path points to a directory, all files inside the directory will be added.
/// If any file does not exist in the working directory, it will be removed from the index.
/// If the file neither exists in the index, an error is returned.
///
/// Files inside the repository directory will not be included.
/// TODO: .gitignore
///
/// IO errors may occur during IO operations. In those cases, an `Error` will be returned.
pub fn add(
    path: &str,
    index_path: &str,
    git_dir_path: &str,
    gitignore_path: &str,
    options: Option<Vec<String>>,
) -> io::Result<()> {
    if is_subpath(path, MGIT) {
        return Ok(());
    }

    if let Some(params) = options {
        if params.len() == 1 && params[0] == "." {
            let current_dir = std::env::current_dir()?;
            let current_dir_str = current_dir.to_str().unwrap();
            
            let file_names: Vec<String> = fs::read_dir(current_dir_str)?
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        e.file_name().into_string().ok()
                    })
                })
                .collect();

                for file_name in file_names {
                    let mut index = Index::load(index_path, git_dir_path, gitignore_path)?;
                    process_file_name(&mut index, &file_name)?;
                    index.write_file()?;
                }
        }
    }else{
        let mut index = Index::load(index_path, git_dir_path, gitignore_path)?;
        process_file_name(&mut index, path)?;
        index.write_file()?;
    }
    Ok(())
}
