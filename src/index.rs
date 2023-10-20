use std::{
    collections::HashMap,
    fs,
    io::{self, Error, Write},
};

use crate::hash_object;
use crate::ignorer::Ignorer;

/// Index is a structure that will help to manage the index file of
/// a repo a.k.a staging area.
///
/// Will have mapped every staged filename and its hash.
/// Will also have a path where it should read and write and a path to
/// a git directory where the new object will be stored.
#[derive(Default)]
pub struct Index {
    map: HashMap<String, String>,
    ignorer: Ignorer,
    path: String,
    git_dir: String,
}

impl Index {
    fn new(index_path: &str, git_dir_path: &str, gitignore_path: &str) -> Self {
        Self {
            map: HashMap::new(),
            ignorer: Ignorer::load(gitignore_path),
            path: String::from(index_path),
            git_dir: String::from(git_dir_path),
        }
    }

    /// This method let the user to create a new index by loading the content
    /// of the given file and a git directory where the objects will be stored.
    ///
    /// May fail if the index path can not be read.
    pub fn load(index_path: &str, git_dir_path: &str, gitignore_path: &str) -> io::Result<Self> {
        let index_content = fs::read_to_string(index_path)?;
        Ok(Self::with(
            &index_content,
            index_path,
            git_dir_path,
            gitignore_path,
        ))
    }

    fn with(
        index_content: &str,
        index_path: &str,
        git_dir_path: &str,
        gitignore_path: &str,
    ) -> Self {
        let mut index = Self::new(index_path, git_dir_path, gitignore_path);
        index.load_content(index_content);
        index
    }

    /// Loads the index in the expected format
    fn load_content(&mut self, index_content: &str) {
        for line in index_content.lines() {
            if let Some((hash, path)) = line.split_once(' ') {
                self.map.insert(path.to_string(), hash.to_string());
            }
        }
    }

    /// Given a path to a file or directory, the index will add, update or remove this path.
    ///
    /// If the path is a directory, then the index will recursively iterate over it until
    /// all files in every sub-directory is added.
    ///
    /// If the file does not exists, then it will be removed from the index.
    pub fn add_path(&mut self, path: &str) -> io::Result<()> {
        if self.ignorer.ignore(path) {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                "The path is ignored by ignore file",
            ));
        }

        match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => self.add_dir(path),
            Ok(_) => {
                let new_hash = hash_object::store_file(path, &self.git_dir)?;
                self.add_file(path, &new_hash)
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

    /// Lets the user to dump the index to a file that can be read un the future by Index
    ///
    /// May fail for an I/O error.
    pub fn write_file(&self) -> io::Result<()> {
        let mut index_file = fs::File::create(&self.path)?;
        for line in &self.map {
            writeln!(index_file, "{} {}", line.1, line.0)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Let the user know if a path is staged or not
    pub fn contains(&self, path: &str) -> bool {
        self.map.contains_key(path)
    }

    /// Given a path, the corresponding hash is returned if the file has been staged.
    ///
    /// If the file has not been staged, then None is returned
    pub fn get_hash(&self, path: &str) -> Option<&String> {
        self.map.get(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_empty() {
        let index = Index::default();
        assert!(index.is_empty())
    }

    #[test]
    fn test_map_keys() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let mut index = Index::default();
        index.load_content(index_content);

        assert!(index.contains("a.txt"));
        assert!(index.contains("b.txt"));
        assert!(index.contains("c.txt"));
        assert!(index.contains("d.txt"));
    }

    #[test]
    fn test_map_values() {
        let index_content = "123456789 a.txt\n12388798 b.txt\n88321767 c.txt\n123817237 d.txt\n";
        let mut index = Index::default();
        index.load_content(index_content);

        assert_eq!(index.get_hash("a.txt"), Some(&"123456789".to_string()));
        assert_eq!(index.get_hash("b.txt"), Some(&"12388798".to_string()));
        assert_eq!(index.get_hash("c.txt"), Some(&"88321767".to_string()));
        assert_eq!(index.get_hash("d.txt"), Some(&"123817237".to_string()));
    }

    #[test]
    fn test_add_new_file() -> io::Result<()> {
        let mut index = Index::default();
        let path = "new.rs";
        let hash = "filehashed";
        index.add_file(path, &hash)?;

        assert!(index.contains(path));
        Ok(())
    }

    #[test]
    fn test_add_updated_file() -> io::Result<()> {
        let mut index = Index::default();
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
        let mut index = Index::default();
        index.load_content(index_content);
        let path = "old.txt";

        assert!(index.contains(path));
        index.remove_file(path)?;

        assert!(!index.contains(path));
        Ok(())
    }

    // For testing an unitialized repository
    fn setup_mgit(git_dir: &str) -> io::Result<()> {
        fs::create_dir_all(format!("{}/objects", git_dir))
    }

    #[test]
    fn test_add_path_file() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(path)?;

        assert!(index.contains(path));
        Ok(())
    }

    #[test]
    fn test_add_path_empty_dir() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let empty_dir_path = "tests/add/dir_to_add/empty";
        fs::create_dir_all(empty_dir_path)?;

        index.add_path(empty_dir_path)?;

        assert!(index.is_empty());
        Ok(())
    }

    #[test]
    fn test_add_path_non_empty_dir() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let dir_path = "tests/add/dir_to_add/non_empty";

        index.add_path(dir_path)?;

        assert!(index.contains("tests/add/dir_to_add/non_empty/a.txt"));
        assert!(index.contains("tests/add/dir_to_add/non_empty/b.txt"));
        Ok(())
    }

    #[test]
    fn test_add_path_non_empty_recursive_dirs() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let dir_path = "tests/add/dir_to_add/recursive";

        index.add_path(dir_path)?;

        assert!(index.contains("tests/add/dir_to_add/recursive/a.txt"));
        assert!(index.contains("tests/add/dir_to_add/recursive/recursive/a.txt"));
        assert!(index.contains("tests/add/dir_to_add/recursive/recursive/recursive/a.txt"));
        Ok(())
    }
}
