// use messi::hash_object::store_file;

use std::{
    collections::HashMap,
    fs,
    io::{self, Error, Write},
};

type Index = HashMap<String, String>;

fn add_dir(path: &str, index: &mut Index) -> io::Result<()> {
    let _ = fs::read_dir(path)?.map(|entry| -> io::Result<()> {
        if let Some(inner_path) = entry?.path().to_str() {
            add_path(inner_path, index)?;
        }
        Ok(())
    });

    Ok(())
}

fn add_file(path: &str, hash: &str, index: &mut Index) -> io::Result<()> {
    index.insert(path.to_string(), hash.to_string());
    Ok(())
}

fn remove_file(path: &str, index: &mut Index) -> io::Result<()> {
    match index.remove(path) {
        Some(_) => Ok(()),
        None => Err(Error::new(
            io::ErrorKind::NotFound,
            format!("Path not found in index: {}. Cannot remove", path),
        )),
    }
}

fn add_path(path: &str, index: &mut Index) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                add_dir(path, index)
            } else {
                let new_hash = "";
                add_file(path, new_hash, index)
            }
        }
        Err(_) => remove_file(path, index),
    }
}

fn map_index(index_content: &str) -> Index {
    index_content
        .lines()
        .map(|line| -> (String, String) {
            let line_split: Vec<&str> = line.splitn(2, ' ').collect();
            if line_split.len() < 2 {
                // tal vez manejar con Err aca, pero en el momento se me complicó un poco
                return (String::new(), String::new());
            }
            // hash filename
            (line_split[1].to_string(), line_split[0].to_string())
        })
        .filter(|line| *line != (String::new(), String::new()))
        .collect()
}

fn read_index() -> io::Result<Index> {
    let index_content = fs::read_to_string(".git/index")?;
    Ok(map_index(&index_content))
}

fn write_index(index: &mut Index) -> io::Result<()> {
    let mut index_file = fs::File::create(".git/index")?;
    for line in index {
        writeln!(index_file, "{} {}", line.1, line.0)?;
    }
    Ok(())
}

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
pub fn add(path: &str) -> io::Result<()> {
    if path.contains(".git/") || path.ends_with(".git") {
        return Ok(());
    }

    let mut index = read_index()?;
    // let gitignore_content = rc::Rc::new(fs::read_to_string(".gitignore")?.lines().collect::<Vec<&str>>());
    add_path(path, &mut index)?;
    write_index(&mut index)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_empty() {
        let index_content = "";
        let index = map_index(index_content);
        assert!(index.is_empty())
    }

    #[test]
    fn test_map_keys() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let index = map_index(index_content);

        assert!(index.contains_key("a.txt"));
        assert!(index.contains_key("b.txt"));
        assert!(index.contains_key("c.txt"));
        assert!(index.contains_key("d.txt"));
    }

    #[test]
    fn test_map_values() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let index = map_index(index_content);

        assert_eq!(index.get("a.txt"), Some(&"123456789".to_string()));
        assert_eq!(index.get("b.txt"), Some(&"12388798".to_string()));
        assert_eq!(index.get("c.txt"), Some(&"88321767".to_string()));
        assert_eq!(index.get("d.txt"), Some(&"123817237".to_string()));
    }

    #[test]
    fn test_add_new_file() -> io::Result<()> {
        let index_content = "";
        let mut index = map_index(index_content);
        let path = "new.rs";
        let hash = "filehashed";
        add_file(path, &hash, &mut index)?;

        assert!(index.contains_key(path));
        Ok(())
    }

    #[test]
    fn test_add_updated_file() -> io::Result<()> {
        let index_content = "";
        let mut index = map_index(index_content);
        let path = "new.rs";
        let hash = "filehashed";
        add_file(path, &hash, &mut index)?;

        let hash = "filehashedupdated";
        add_file(path, &hash, &mut index)?;
        assert_eq!(index.get(path), Some(&hash.to_string()));
        Ok(())
    }

    #[test]
    fn test_remove_file() -> io::Result<()> {
        let index_content = "hashed old.txt";
        let mut index = map_index(index_content);
        let path = "old.txt";

        assert!(index.contains_key(path));
        remove_file(path, &mut index)?;
        assert!(!index.contains_key(path));
        Ok(())
    }
}
