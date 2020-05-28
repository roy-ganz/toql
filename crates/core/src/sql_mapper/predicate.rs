

use crate::sql_mapper::predicate_options::PredicateOptions;
use std::sync::Arc;
use crate::predicate_handler::PredicateHandler;
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Predicate {
    pub(crate) expression: SqlExpr,
    pub(crate) handler: Arc<dyn PredicateHandler + Send + Sync>, // Handler to create clauses
    //pub(crate) sql_aux_param_names: Vec<String>, // aux params in predicate statement or ? in correct order
    pub(crate) options: PredicateOptions,
}
