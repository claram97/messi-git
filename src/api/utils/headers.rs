use std::collections::HashMap;

/// A struct that holds the headers of a request.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Headers(HashMap<String, String>);

impl Headers {
    /// Create a new Headers.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a header into the headers map.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the header to be added.
    /// * `value` - A string slice that holds the value of the header to be added.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_lowercase(), value.to_lowercase());
    }

    /// Get a header from the headers map.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the header to be retrieved.
    pub fn get(&self, key: &str) -> Option<&str> {
        match self.0.get(key.to_lowercase().as_str()) {
            Some(value) => Some(value),
            None => None,
        }
    }

    /// Add a header to the headers map.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that holds the header and its value to be added.
    pub fn add(&mut self, line: &str) {
        let mut key_value = line.split(": ");
        let key = key_value.next().unwrap_or_default();
        let value = key_value.next().unwrap_or_default();
        self.insert(key, value);
    }
}

impl From<Vec<&str>> for Headers {
    fn from(v: Vec<&str>) -> Self {
        let mut headers = Self::new();
        for line in v {
            headers.add(line)
        }
        headers
    }
}

impl std::fmt::Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "\r\n")?;
            return Ok(());
        }
        for (key, value) in &self.0 {
            write!(f, "\r\n{}: {}", key, value)?;
        }
        Ok(())
    }
}
