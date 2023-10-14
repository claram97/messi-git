// use crate::hash_object::store_file;

const MGIT: &str = ".mgit";
const OPTIONS_ALL: &str = "-u";

use std::io;

use crate::ignorer::is_subpath;
use crate::index::Index;

/// This function receives a path to append/remove from the staging area
///
/// If the path points to a directory, then all files inside will be added
///
/// If any file does not exists in the working area, then will be removed from the
/// staging area.
/// If the file neither exists in the staging area, then an error is returned.
///
/// Files inside repository directory will not be included.
/// TODO: .gitignore
///
/// IO errors may occurr while doing IO operations. In that cases, Error will be returned.
pub fn add(path: &str, options: Option<Vec<String>>) -> io::Result<()> {
    if is_subpath(path, MGIT) {
        return Ok(());
    }

    if let Some(params) = options {
        if params.contains(&OPTIONS_ALL.to_string()) {
            return add(".", None);
        }
    }

    let mut index = Index::new();
    index.load_index()?;
    index.add_path(path)?;
    index.write_file()?;

    Ok(())
}
