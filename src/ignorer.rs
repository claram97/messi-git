use std::fs;

const MGIT_IGNORE: &str = ".mgitignore";

/// This is a helper structure that will help some git commands
/// to know if a path has to be ignored or not according to
/// the content of git ignore file.
#[derive(Default)]
pub struct Ignorer {
    paths: Vec<String>,
}

impl Ignorer {
    /// This method loads the git ignore file and returns an
    /// Ignorer ready to use
    pub fn load() -> Self {
        match fs::read_to_string(MGIT_IGNORE) {
            Ok(file) => Self {
                paths: file.lines().map(str::to_string).collect(),
            },
            Err(_) => Self { paths: Vec::new() },
        }
    }

    /// This method will decide whether a path has to be ignored or not
    pub fn ignore(&self, path: &str) -> bool {
        for ignored in &self.paths {
            if is_subpath(path, ignored) {
                return true;
            };
        }
        false
    }
}

fn get_subpaths(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|subpath| !subpath.is_empty())
        .collect()
}

/// This is a helper function that can be util also outside Ignorer
/// Checks if a path is a subpath of the other
pub fn is_subpath(subpath: &str, path: &str) -> bool {
    let path_parent: Vec<&str> = get_subpaths(path);
    let path_child: Vec<&str> = get_subpaths(subpath);

    for i in 0..path_parent.len() {
        match (path_parent.get(i), path_child.get(i)) {
            (Some(subpath_parent), Some(subpath_child)) => {
                if subpath_parent != subpath_child {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        assert!(is_subpath("src/a", "src"));
    }

    #[test]
    fn test_2() {
        assert!(is_subpath("src/a", "src/"));
    }

    #[test]
    fn test_3() {
        assert!(is_subpath("src/a", "/src/"));
    }

    #[test]
    fn test_4() {
        assert!(is_subpath("src/a/a/a/d/d/w/e/e", "src/a/"));
    }

    #[test]
    fn test_5() {
        assert!(is_subpath("src/data.txt", "src/data.txt"));
    }

    #[test]
    fn test_6() {
        assert!(!is_subpath("src/data", "src/data/data.txt"));
    }

    #[test]
    fn test_7() {
        assert!(is_subpath("src/data/data.txt", "src/data"));
    }

    #[test]
    fn test_8() {
        let mut ignorer = Ignorer::default();
        ignorer.paths.push("init.rs".to_string());
        assert!(!ignorer.ignore("src/init.rs"));
    }
}
