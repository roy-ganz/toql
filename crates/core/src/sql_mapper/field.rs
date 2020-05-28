


use crate::sql_expr::SqlExpr;
use std::sync::Arc;
use crate::field_handler::FieldHandler;

use crate::sql_mapper::field_options::FieldOptions;


#[derive(Debug)]
#[allow(dead_code)] 
pub(crate) enum FilterType {
    Where,
    None,
}


#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) options: FieldOptions,                        // Options
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>, // Handler to build select and filter
    //pub(crate) subfields: bool, // Field name has subfields separated by underscore
    pub(crate) expression: SqlExpr, // Column name or SQL expression
}