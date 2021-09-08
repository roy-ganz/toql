use crate::{error::ToqlError, query::field_path::FieldPath, sql_expr::SqlExpr, sql_arg::SqlArg};

// Trait is implemented for structs that can insert
pub trait TreeInsert {
    fn columns<'a, I>(descendents: &mut I) -> Result<SqlExpr, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
    fn values<'a, I>(
        &self,
        descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        key_limits: Option<&[Vec<SqlArg>]>,
        values: &mut crate::sql_expr::SqlExpr,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
