//! A list of path names.

/// The struct hold a list of path names.
/// Users of the Toql library should use the paths! macro to
/// build it. The `paths!` macro provides compile time garanties.
/// Unlike [Query](crate::query::Query) Paths is not typesafe.
/// This will be improved in the future.
pub struct Paths {
    pub list: Vec<String>,
}

impl Paths {
    pub fn top() -> Self {
        Self::from(vec![])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Paths { list: fields }
    }
    pub fn into_inner(self) -> Vec<String> {
        self.list
    }
}
