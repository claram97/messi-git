#[derive(Default, PartialEq)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub fetch: String,
}

impl Remote {
    pub fn new(name: String, url: String, fetch: String) -> Remote {
        Remote { name, url, fetch }
    }
}
