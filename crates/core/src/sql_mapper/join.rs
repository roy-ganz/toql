
use super::join_options::JoinOptions;

#[derive(Debug, PartialEq)]
pub enum JoinType {
    Left,
    Inner,
}

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) join_type: JoinType,   // LEFT JOIN ...
    pub(crate) aliased_table: String, // Table t0
    pub(crate) on_predicate: String,  // ON ..
    pub(crate) options: JoinOptions,
    pub(crate) sql_aux_param_names: Vec<String>, // aux params in ON clause
}