
use crate::query::field_path::Descendents;
use crate::sql_expr::SqlExpr;
use crate:: error :: ToqlError;

pub trait TreePredicate {
    fn predicate<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        field: &str,
        predicate: &mut SqlExpr,
    ) -> Result<(), ToqlError>;
}
