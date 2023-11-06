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
    /// Create a new instance of index with the specified paths.
    ///
    /// This function initializes a new index and sets its internal fields based
    /// on the provided `index_path` and `git_dir_path`. The `map` field is initialized
    /// as an empty `HashMap`, the `ignorer` field is loaded from some source, and the
    /// `path` and `git_dir` fields are set from the input parameters.
    ///
    /// # Arguments
    ///
    /// * `index_path` - A string slice representing the path to the index.
    /// * `git_dir_path` - A string slice representing the path to the Git directory.
    ///
    /// # Returns
    ///
    /// A new instance of index.
    pub fn new(index_path: &str, git_dir_path: &str, gitignore_path: &str) -> Self {
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

    /// Create a new instance of index and populate it with the provided content.
    ///
    /// This function initializes a new index by calling `new` with the given `index_path`
    /// and `git_dir_path`. It then loads the content from the `index_content` into the newly
    /// created instance using the `load_content` method.
    ///
    /// # Arguments
    ///
    /// * `index_content` - A string slice containing the content to load into the index.
    /// * `index_path` - A string slice representing the path to the index.
    /// * `git_dir_path` - A string slice representing the path to the Git directory.
    ///
    /// # Returns
    ///
    /// A new instance of index populated with the content.
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

    /// Recursively add all files and subdirectories under the specified directory to the index.
    ///
    /// This function iterates through the contents of the directory located at the provided `path`
    /// and recursively adds all files and subdirectories to the index using the `add_path` method.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the path of the directory to add to the index.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating the result of the operation. It returns `Ok(())` on success and an
    /// `Err` variant if any I/O errors occur during the process.
    fn add_dir(&mut self, path: &str) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            if let Some(inner_path) = entry?.path().to_str() {
                self.add_path(inner_path)?;
            }
        }

        Ok(())
    }

    /// Add a file to the index with its corresponding hash.
    ///
    /// This function adds a file identified by the provided `path` to the index. The file is associated
    /// with its `hash`, which represents its content's hash or unique identifier.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the path of the file to be added to the index.
    /// * `hash` - A string slice representing the hash or unique identifier of the file's content.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating the result of the operation. It returns `Ok(())` on success, indicating
    /// that the file was added to the index successfully.
    pub fn add_file(&mut self, path: &str, hash: &str) -> io::Result<()> {
        self.map.insert(path.to_string(), hash.to_string());
        Ok(())
    }

    /// Remove a file from the index by its path.
    ///
    /// This function removes a file from the index based on its `path`. If the file is found in the index,
    /// it is removed; otherwise, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the path of the file to be removed from the index.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating the result of the operation. If the file is successfully removed from the index,
    /// it returns `Ok(())`. If the file is not found in the index, an `Err` with an `io::ErrorKind::NotFound` error
    /// is returned, along with an error message indicating that the path was not found in the index.
    pub fn remove_file(&mut self, path: &str) -> io::Result<()> {
        match self.map.remove(path) {
            Some(_) => Ok(()),
            None => Err(Error::new(
                io::ErrorKind::NotFound,
                format!("Path not found in index: {}. Cannot remove", path),
            )),
        }
    }

    /// Load an index from a file if it exists at the specified path.
    ///
    /// This function attempts to load an index from a file at the specified `index_path`. If the file exists
    /// and can be successfully loaded, the function returns `Some(index)`. If the file does not exist or cannot be loaded,
    /// it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `index_path` - A string slice representing the path to the index file.
    /// * `git_dir_path` - A string slice representing the path to the Git directory.
    ///
    /// # Returns
    ///
    /// An `io::Result` containing an optional `Index`. If the file at `index_path` exists and can be loaded, the result is `Ok(Some(index))`.
    /// If the file does not exist or loading fails, the result is `Ok(None)`.
    pub fn load_from_path_if_exists(
        index_path: &str,
        git_dir_path: &str,
        git_ignore_path: &str,
    ) -> io::Result<Option<Index>> {
        if let Ok(metadata) = fs::metadata(index_path) {
            if metadata.is_file() {
                return Index::load(index_path, git_dir_path, git_ignore_path).map(Some);
            }
        }
        Ok(None)
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

    /// Returns an iterator over the key-value pairs in the map.
    ///
    /// This function returns an iterator that allows you to iterate over the key-value pairs in the map.
    ///
    /// # Returns
    /// An iterator over the key-value pairs in the map.
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, String> {
        self.map.iter()
    }

    /// Checks if a path should be ignored based on the provided `ignorer`.
    ///
    /// This function checks if a given `path` should be ignored by using the provided `ignorer`.
    ///
    /// # Arguments
    /// - `path`: A reference to a string representing the path to be checked.
    ///
    /// # Returns
    /// `true` if the path should be ignored, otherwise `false`.
    pub fn path_should_be_ignored(&self, path: &str) -> bool {
        self.ignorer.ignore(path)
    }

    #[cfg(test)]
    /// Check if the index is empty.
    ///
    /// This function returns `true` if the index is empty, meaning it contains no entries, and `false` otherwise.
    ///
    /// # Returns
    ///
    /// A boolean value, `true` if the index is empty, and `false` otherwise.
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

    /// Test loading the index from an existing path.
    ///
    /// This test checks whether the `load_from_path_if_exists` function correctly loads an index from an
    /// existing file path. It creates a temporary test index file, populates it with some test content,
    /// and then attempts to load the index from that path. It asserts that the loaded index is not empty.
    ///
    /// The test performs the following steps:
    /// 1. Create an empty `Index` instance.
    /// 2. Define a test index file path and a test Git directory path.
    /// 3. Populate the test index file with sample content.
    /// 4. Attempt to load the index from the test index file path.
    /// 5. Assert that the loaded index is not empty.
    /// 6. Clean up by removing the test index file and Git directory.
    ///
    /// # Panics
    ///
    /// The test will panic if the loaded index is empty, indicating a failure.
    #[test]
    fn test_load_from_path_if_exists_with_existing_index() {
        let _index = Index::default();
        let test_index_path = "test_index.index";
        let test_git_dir_path = "test_git";

        let index_content = "hash_de_prueba archivo_prueba.txt\n";
        fs::write(test_index_path, index_content).unwrap();

        let result = Index::load_from_path_if_exists(test_index_path, test_git_dir_path, "");
        if let Ok(Some(index)) = result {
            assert!(index.contains("archivo_prueba.txt"));
        } else {
            panic!("Se esperaba Some(Index), pero se obtuvo otro resultado.");
        }

        fs::remove_file(test_index_path).ok();
        fs::remove_dir_all(test_git_dir_path).ok();
    }

    /// Test loading the index from a non-existing path.
    ///
    /// This test checks whether the `load_from_path_if_exists` function handles the case when the index
    /// file does not exist at the specified path. It attempts to load the index from a non-existing file
    /// path and asserts that the result is `None`, indicating that the index was not found.
    ///
    /// The test performs the following steps:
    /// 1. Create an empty `Index` instance.
    /// 2. Define a test index file path and a test Git directory path.
    /// 3. Attempt to load the index from the non-existing test index file path.
    /// 4. Assert that the result is `None`.
    /// 5. Clean up by removing any temporary files or directories.
    ///
    /// # Panics
    ///
    /// The test will panic if the loaded result is not `None, indicating a failure.
    #[test]
    fn test_load_from_path_if_exists_with_non_existing_index() {
        let _index = Index::default();
        let test_index_path = "test_index1.index";
        let test_git_dir_path = "test_git";

        let result = Index::load_from_path_if_exists(test_index_path, test_git_dir_path, "");
        if let Ok(None) = result {
        } else {
            panic!("Se esperaba None, pero se obtuvo otro resultado.");
        }

        fs::remove_file(test_index_path).ok();
        fs::remove_dir_all(test_git_dir_path).ok();
    }

    /// Test checking if the index is empty.
    ///
    /// This test verifies the `is_empty` method's correctness by creating an empty `Index` instance
    /// and then asserting that the `is_empty` method correctly identifies it as empty.
    ///
    /// The test performs the following steps:
    /// 1. Create an empty `Index` instance.
    /// 2. Call the `is_empty` method on the index.
    /// 3. Assert that the method returns `true`, indicating that the index is empty.
    ///
    /// # Panics
    ///
    /// The test will panic if the index is not considered empty by the `is_empty` method.
    #[test]
    fn test_map_empty() {
        let index = Index::default();
        assert!(index.is_empty())
    }

    /// Test checking the presence of keys in the index.
    ///
    /// This test verifies that the `contains` method correctly identifies the presence of keys in the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance and load it with specific content.
    /// 2. Use the `contains` method to check the presence of multiple keys.
    /// 3. Assert that the method returns `true` for each key that should be present.
    ///
    /// # Panics
    ///
    /// The test will panic if any of the keys are not correctly identified as present in the index.
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

    /// Test verifying the correctness of hash values in the index.
    ///
    /// This test checks that the `get_hash` method correctly returns the expected hash values for specific keys in the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance and load it with specific content.
    /// 2. Use the `get_hash` method to retrieve the hash values for various keys.
    /// 3. Assert that the returned hash values match the expected values.
    ///
    /// # Panics
    ///
    /// The test will panic if any of the hash values returned by the `get_hash` method do not match the expected values.
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

    /// Test for adding a new file to the index.
    ///
    /// This test verifies the behavior of the `add_file` method when adding a new file to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an empty `Index` instance.
    /// 2. Use the `add_file` method to add a new file entry with a path and hash.
    /// 3. Assert that the file has been successfully added to the index using the `contains` method.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
    #[test]
    fn test_add_new_file() -> io::Result<()> {
        let mut index = Index::default();
        let path = "new.rs";
        let hash = "filehashed";
        index.add_file(path, &hash)?;

        assert!(index.contains(path));
        Ok(())
    }

    /// Test for adding an updated file to the index.
    ///
    /// This test verifies the behavior of the `add_file` method when adding an updated file to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an empty `Index` instance.
    /// 2. Use the `add_file` method to add a new file entry with a path and initial hash.
    /// 3. Update the file entry by using the `add_file` method with a new hash for the same path.
    /// 4. Assert that the file's hash has been successfully updated in the index using the `get_hash` method.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
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

    /// Test for removing a file from the index.
    ///
    /// This test verifies the behavior of the `remove_file` method when removing a file from the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance and load it with a predefined content that includes a file entry to be removed.
    /// 2. Check if the file entry is initially present in the index using the `contains` method.
    /// 3. Use the `remove_file` method to remove the specified file entry.
    /// 4. Assert that the file entry is no longer present in the index by checking with the `contains` method.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file removal process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
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

    /// Test for adding files by path to the index.
    ///
    /// This test verifies the behavior of the `add_path` method when adding files from a specified path to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance with a specific Git directory and a repository path.
    /// 2. Set up the Git repository using a helper function (`setup_mgit`) to prepare the environment.
    /// 3. Specify a file path to add to the index.
    /// 4. Use the `add_path` method to add files from the specified path to the index.
    /// 5. Assert that the specified file is present in the index by checking with the `contains` method.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
    #[test]
    fn test_add_path_file() -> io::Result<()> {
        let mut index = Index::new("", ".mgit", "");
        setup_mgit(".mgit")?;

        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(path)?;

        assert!(index.contains(path));
        Ok(())
    }

    /// Test for adding files from an empty directory to the index.
    ///
    /// This test verifies the behavior of the `add_path` method when adding files from an empty directory to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance with a specific Git directory and a repository path.
    /// 2. Set up the Git repository using a helper function (`setup_mgit`) to prepare the environment.
    /// 3. Create an empty directory and specify its path.
    /// 4. Use the `add_path` method to add files from the specified empty directory to the index.
    /// 5. Assert that the index remains empty after the addition.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
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

    /// Test for adding files from a non-empty directory to the index.
    ///
    /// This test verifies the behavior of the `add_path` method when adding files from a non-empty directory to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance with a specific Git directory and a repository path.
    /// 2. Set up the Git repository using a helper function (`setup_mgit`) to prepare the environment.
    /// 3. Specify the path to a non-empty directory that contains multiple files.
    /// 4. Use the `add_path` method to add files from the specified directory to the index.
    /// 5. Assert that the index contains the expected files from the directory.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
    #[ignore]
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

    /// Test for adding files from nested directories to the index.
    ///
    /// This test verifies the behavior of the `add_path` method when adding files from nested directories to the index.
    ///
    /// The test performs the following steps:
    /// 1. Create an `Index` instance with a specific Git directory and a repository path.
    /// 2. Set up the Git repository using a helper function (`setup_mgit`) to prepare the environment.
    /// 3. Specify the path to a directory that contains nested directories with files.
    /// 4. Use the `add_path` method to add files from the specified directory and its subdirectories to the index.
    /// 5. Assert that the index contains the expected files from the nested directories.
    ///
    /// # Errors
    ///
    /// The test will return an error if any of the assertions fail or if there is an error during the file addition process.
    ///
    /// # Panics
    ///
    /// The test may panic if any of the assertions fail.
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
