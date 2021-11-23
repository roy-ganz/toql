use super::predicate_options::PredicateOptions;
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Predicate {
    pub(crate) expression: SqlExpr,
    pub(crate) options: PredicateOptions,
}
