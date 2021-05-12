use crate::error::ToqlError;
use crate::query::field_path::{Descendents, FieldPath};
use crate::sql_arg::SqlArg;

pub trait TreePredicate {
    fn columns<'a, I>(&self, descendents: &mut I) -> Result<Vec<String>, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;

    fn args<'a, I>(&self, descendents: &mut I, args: &mut Vec<SqlArg>) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
