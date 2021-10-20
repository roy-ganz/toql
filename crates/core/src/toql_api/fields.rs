//! A list of field names.

/// The struct hold a list of field names.
/// Users of the Toql library should use the fields! macro to
/// build it. The `fields!` macro provides compile time garanties.
/// Unlike [Query](crate::query::Query) Fields is not typesafe.
/// This will be improved in the future.
pub struct Fields {
    pub list: Vec<String>,
}

impl Fields {
    pub fn top() -> Self {
        Self::from(vec!["*".to_string()])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Fields { list: fields }
    }

    pub fn into_inner(self) -> Vec<String> {
        self.list
    }
}
