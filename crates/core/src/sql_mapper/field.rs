
use super::field_options::FieldOptions;
use crate::field_handler::FieldHandler;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) options: FieldOptions,                        // Options
    pub(crate) filter_type: FilterType,                      // Filter on where or having clause
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) subfields: bool, // Target name has subfields separated by underscore
    pub(crate) expression: String, // Column name or SQL expression
    pub(crate) sql_aux_param_names: Vec<String>, //  Extracted from <aux_param>
}


#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}