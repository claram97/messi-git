use std::collections::HashMap;

/// A struct that holds the query strings of a request.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct QueryString(HashMap<String, String>);

impl QueryString {
    /// Create a new QueryString.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a query string into the query strings map.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the query string to be added.
    /// * `value` - A string slice that holds the value of the query string to be added.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_string(), value.to_string());
    }

    /// Get a query string from the query strings map.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the query string to be retrieved.
    pub fn get(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(value) => Some(value),
            None => None,
        }
    }

    /// Add a query string to the query strings map.
    ///
    /// # Arguments
    ///
    /// * `pair` - A string slice that holds the query string and its value to be added.
    fn add(&mut self, pair: &str) {
        let mut key_value = pair.split('=');
        let key = key_value.next().unwrap_or_default();
        let value = key_value.next().unwrap_or_default();
        self.insert(key, value);
    }
}

impl From<&str> for QueryString {
    fn from(s: &str) -> Self {
        let mut qs = Self::new();
        for pair in s.split('&') {
            qs.add(pair);
        }
        qs
    }
}
