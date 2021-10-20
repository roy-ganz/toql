//! Trait to build UPDATE SQL statements.
use crate::{error::ToqlError, query::field_path::FieldPath, sql_expr::SqlExpr};

/// The trait allows to build the UPDATE SQL statement for nested structs.
///
/// Trait is implemented by the Toql derive for structs that can update.
pub trait TreeUpdate {
    fn update<'a, I>(
        &self,
        descendents: I,
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<SqlExpr>,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone;
}
