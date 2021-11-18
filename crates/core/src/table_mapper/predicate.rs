use super::predicate_options::PredicateOptions;
use crate::predicate_handler::PredicateHandler;
use crate::sql_expr::SqlExpr;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Predicate {
    pub(crate) expression: SqlExpr,
    pub(crate) options: PredicateOptions,
}
