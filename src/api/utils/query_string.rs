use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct QueryString(HashMap<String, String>);

impl QueryString {
    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(value) => Some(value),
            None => None,
        }
    }
}

impl From <&str> for QueryString {
    fn from(s: &str) -> Self {
        let mut map = HashMap::new();
        for pair in s.split("&") {
            let mut key_value = pair.split("=");
            let key = key_value.next().unwrap_or_default().to_string();
            let value = key_value.next().unwrap_or_default().to_string();
            map.insert(key, value);
        }
        Self(map)
    }
}