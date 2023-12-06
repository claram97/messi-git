use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Headers(HashMap<String, String>);

impl Headers {
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

impl From<Vec<&str>> for Headers {
    fn from(v: Vec<&str>) -> Self {
        let mut map = HashMap::new();
        for line in v {
            let mut key_value = line.split(": ");
            let key = key_value.next().unwrap_or_default().to_string();
            let value = key_value.next().unwrap_or_default().to_string();
            map.insert(key, value);
        }
        Self(map)
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
