//! Trait to build key predicate.
use crate::{error::ToqlError, query::field_path::FieldPath, sql_arg::SqlArg};

/// The trait allows to build the a key predicate for nested structs.
pub trait TreePredicate {
    /// Return the key column names
    fn columns<'a, I>(descendents: I) -> Result<Vec<String>, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
    
    /// Return the key column values
    fn args<'a, I>(&self, descendents: I, args: &mut Vec<SqlArg>) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone;
}
