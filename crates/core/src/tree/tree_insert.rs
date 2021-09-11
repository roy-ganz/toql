use crate::{error::ToqlError, query::field_path::FieldPath, sql_expr::SqlExpr};

// Trait is implemented for structs that can insert
pub trait TreeInsert {
    fn columns<'a, I>(descendents: &mut I) -> Result<SqlExpr, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
    fn values<'a,'b, I, J>(
        &self,
        descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        should_insert: &mut J,
        values: &mut crate::sql_expr::SqlExpr,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
        J: Iterator<Item = &'b bool>;
}
