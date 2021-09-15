use crate::{error::ToqlError, query::field_path::FieldPath, sql_expr::SqlExpr, sql_arg::SqlArg};

// Trait is implemented for structs that can update
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
