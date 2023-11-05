#[derive(Default, PartialEq)]
pub struct Branch {
    pub name: String,
    pub remote: String,
    pub merge: String,
}

impl Branch {
    /// The `Branch` struct holds information about a Git branch, including its name, the remote reference it tracks,
    /// and the configuration for merging changes. This constructor is used to create new instances of the `Branch`
    /// struct with the specified attributes.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the branch.
    /// * `remote`: The name of the remote reference this branch tracks.
    /// * `merge`: The configuration specifying how changes are merged into this branch.
    ///
    /// # Returns
    ///
    /// Returns a new `Branch` instance with the provided attributes.
    ///
    pub fn new(name: String, remote: String, merge: String) -> Branch {
        Branch {
            name,
            remote,
            merge,
        }
    }
}
