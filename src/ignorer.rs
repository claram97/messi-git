use std::fs;

const MGIT_IGNORE: &str = ".mgitignore";

pub struct Ignorer {
    paths: Vec<String>,
}

impl Ignorer {
    pub fn load() -> Self {
        match fs::read_to_string(MGIT_IGNORE) {
            Ok(file) => Self {
                paths: file.lines().map(str::to_string).collect(),
            },
            Err(_) => Self { paths: Vec::new() },
        }
    }

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

pub fn is_subpath(subpath: &str, path: &str) -> bool {
    let path_parent: Vec<&str> = get_subpaths(path);
    let path_child: Vec<&str> = get_subpaths(subpath);

    for i in 0..path_parent.len() {
        if path_parent[i] != path_child[i] {
            return false;
        }
    }
    true
}

// hacer tests de integracion con file real
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
}
