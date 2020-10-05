use super::predicate_options::PredicateOptions;
use crate::predicate_handler::PredicateHandler;
use crate::sql_expr::SqlExpr;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Predicate {
    pub(crate) expression: SqlExpr,
    pub(crate) handler: Arc<dyn PredicateHandler + Send + Sync>, // Handler to create clauses
    pub(crate) options: PredicateOptions,
}
