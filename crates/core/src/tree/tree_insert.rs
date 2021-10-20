//! Trait to build INSERT SQL statement.
use crate::{error::ToqlError, query::field_path::FieldPath, sql_expr::SqlExpr};

/// The trait allows to build the INSERT SQL statement for nested structs.
pub trait TreeInsert {
    /// Return columns of struct located at `descendents`.
    fn columns<'a, I>(descendents: I) -> Result<SqlExpr, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
    /// Return values of structs located at `descendents`.
    /// The `should_insert` argument allows the method to only insert a part of the collection.
    /// This is needed to deal with newly added merges.
    fn values<'a, 'b, I, J>(
        &self,
        descendents: I,
        roles: &std::collections::HashSet<String>,
        should_insert: &mut J,
        values: &mut crate::sql_expr::SqlExpr,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>;
}
