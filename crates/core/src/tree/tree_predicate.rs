use crate::error::ToqlError;
use crate::query::field_path::Descendents;
use crate::sql_expr::SqlExpr;

pub trait TreePredicate {
    fn predicate<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        field: &str,
        predicate: &mut SqlExpr,
    ) -> Result<(), ToqlError>;
}
