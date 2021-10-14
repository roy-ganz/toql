//! Combines multiple aux parameters in a lightweight way.
/// It is used to combine multiple aux parameters, that may come
/// from the [Context](backend/context/struct.Context),
/// from a [Query](query/struct.Query) or a predicate or field mapping.
use crate::sql_arg::SqlArg;
use std::collections::HashMap;

#[derive(Debug)]
///
pub struct ParameterMap<'a>(&'a [&'a HashMap<String, SqlArg>]);

impl<'a> ParameterMap<'a> {
    pub fn new(params: &'a [&'a HashMap<String, SqlArg>]) -> Self {
        ParameterMap(params)
    }
    /// Returns true, if the map contains a parameter with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|m| m.contains_key(name))
    }

    // Searches the map for a given parameter and returns its value or None.
    pub fn get(&self, name: &str) -> Option<&'a SqlArg> {
        for p in self.0 {
            if let Some(p) = p.get(name) {
                return Some(p);
            }
        }
        None
    }
}
