#[derive(Default)]
#[derive(PartialEq)]
pub struct Branch {
    pub name : String,
    pub remote : String,
    pub merge : String,
}

impl Branch {
    pub fn new(name : String, remote: String, merge: String) -> Branch {
        Branch {
            name,
            remote,
            merge,
        }
    }
}