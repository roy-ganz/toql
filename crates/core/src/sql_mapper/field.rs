
use super::field_options::FieldOptions;
use crate::field_handler::FieldHandler;
use std::sync::Arc;
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) options: FieldOptions,                        // Options
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) expression: SqlExpr, // Column name or SQL expression
 
}


#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}