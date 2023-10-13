// use messi::hash_object::store_file;

const MGIT: &str = ".mgit";
const MGIT_DIR: &str = ".mgit/";
const INDEX_PATH: &str = ".mgit/index";
// const MGIT_IGNORE: &str = ".mgitignore";

use std::{
    collections::HashMap,
    fs,
    io::{self, Error, Write},
};

struct Index  {
    map: HashMap<String, String>
}

impl Index {

    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    #[cfg(test)]
    fn with(index_content: &str) -> Self {
        let mut index = Self::new();
        index.load_content(index_content);
        index
    }

    fn load_index(&mut self) -> io::Result<()> {
        let index_content = fs::read_to_string(INDEX_PATH)?;
        self.load_content(&index_content);
        Ok(())
    }

    fn load_content(&mut self, index_content: &str) {

        for line in index_content.lines() {
            let line_split: Vec<&str> = line.splitn(2, ' ').collect();
            if line_split.len() < 2 {
                continue;
            }
            self.map.insert(line_split[1].to_string(), line_split[0].to_string());
        }
    }

    fn add_path(&mut self, path: &str) -> io::Result<()> {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => self.add_dir(path),
            Ok(_) => {
                let new_hash = "";
                self.add_file(path, new_hash)
            }
            Err(_) => self.remove_file(path),
        }
    }

    fn add_dir(&mut self, path: &str) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            if let Some(inner_path) = entry?.path().to_str() {
                self.add_path(inner_path)?;
            }
        }
    
        Ok(())
    }

    fn add_file(&mut self, path: &str, hash: &str) -> io::Result<()> {
        self.map.insert(path.to_string(), hash.to_string());
        Ok(())
    }

    fn remove_file(&mut self, path: &str) -> io::Result<()> {
        match self.map.remove(path) {
            Some(_) => Ok(()),
            None => Err(Error::new(
                io::ErrorKind::NotFound,
                format!("Path not found in index: {}. Cannot remove", path),
            )),
        }
    }

    fn write_file(&self) -> io::Result<()> {
        let mut index_file = fs::File::create(INDEX_PATH)?;
        for line in &self.map {
            writeln!(index_file, "{} {}", line.1, line.0)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[cfg(test)]
    fn contains(&self, path: &str) -> bool {
        self.map.contains_key(path)
    }

    #[cfg(test)]
    fn get_hash(&self, path: &str) -> Option<&String> {
        self.map.get(path)
    }
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
    if path.contains(MGIT_DIR) || path == MGIT {
        return Ok(());
    }

    let mut index = Index::new();
    index.load_index()?;
    index.add_path(path)?;
    index.write_file()?;

    // let mut index = read_index()?;
    // // let gitignore_content = rc::Rc::new(fs::read_to_string(".gitignore")?.lines().collect::<Vec<&str>>());
    // add_path(path, &mut index)?;
    // write_index(&mut index)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_empty() {
        let index = Index::new();
        assert!(index.is_empty())
    }

    #[test]
    fn test_map_keys() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let index = Index::with(index_content);

        assert!(index.contains("a.txt"));
        assert!(index.contains("b.txt"));
        assert!(index.contains("c.txt"));
        assert!(index.contains("d.txt"));
    }

    #[test]
    fn test_map_values() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let index = Index::with(index_content);

        assert_eq!(index.get_hash("a.txt"), Some(&"123456789".to_string()));
        assert_eq!(index.get_hash("b.txt"), Some(&"12388798".to_string()));
        assert_eq!(index.get_hash("c.txt"), Some(&"88321767".to_string()));
        assert_eq!(index.get_hash("d.txt"), Some(&"123817237".to_string()));
    }

    #[test]
    fn test_add_new_file() -> io::Result<()> {
        let mut index = Index::new();
        let path = "new.rs";
        let hash = "filehashed";
        index.add_file(path, &hash)?;

        assert!(index.contains(path));
        Ok(())
    }

    #[test]
    fn test_add_updated_file() -> io::Result<()> {
        let mut index = Index::new();
        let path = "new.rs";
        let hash = "filehashed";
        index.add_file(path, &hash)?;

        let hash = "filehashedupdated";
        index.add_file(path, &hash)?;
        assert_eq!(index.get_hash(path), Some(&hash.to_string()));
        Ok(())
    }

    #[test]
    fn test_remove_file() -> io::Result<()> {
        let index_content = "hashed old.txt";
        let mut index = Index::with(index_content);
        let path = "old.txt";

        assert!(index.contains(path));
        index.remove_file(path)?;

        assert!(!index.contains(path));
        Ok(())
    }

    #[test]
    fn test_add_path_file() -> io::Result<()> {
        let mut index = Index::new();

        let path = "tests/dir_to_add/non_empty/a.txt";

        index.add_path(path)?;

        assert!(index.contains(path));
        Ok(())
    }

    #[test]
    fn test_add_path_empty_dir() -> io::Result<()> {
        let mut index = Index::new();
        let empty_dir_path = "tests/dir_to_add/empty";

        index.add_path(empty_dir_path)?;

        assert!(index.is_empty());
        Ok(())
    }

    #[test]
    fn test_add_non_empty_dir() -> io::Result<()> {
        let mut index = Index::new();
        let dir_path = "tests/dir_to_add/non_empty";

        index.add_path(dir_path)?;

        assert!(index.contains("tests/dir_to_add/non_empty/a.txt"));
        assert!(index.contains("tests/dir_to_add/non_empty/b.txt"));
        Ok(())
    }

    #[test]
    fn test_add_non_empty_recursive_dirs() -> io::Result<()> {
        let mut index = Index::new();
        let dir_path = "tests/dir_to_add/recursive";

        index.add_path(dir_path)?;

        assert!(index.contains("tests/dir_to_add/recursive/a.txt"));
        assert!(index.contains("tests/dir_to_add/recursive/recursive/a.txt"));
        assert!(index.contains("tests/dir_to_add/recursive/recursive/recursive/a.txt"));
        Ok(())
    }
}