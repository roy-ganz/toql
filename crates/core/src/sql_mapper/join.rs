


use crate::sql_expr::SqlExpr;
use crate::sql_mapper::join_options::JoinOptions;

#[derive(Debug, PartialEq)]
pub(crate) enum JoinType {
    Left,
    Inner,
}

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) joined_mapper: String, // Mapper to join (Provides table name and alias) 
    pub(crate) join_expression: SqlExpr,
    /* pub(crate) join_type: JoinType,   // LEFT JOIN ...
    pub(crate) on_predicate: String,  // ON .. */
    pub(crate) options: JoinOptions,
    //pub(crate) sql_aux_param_names: Vec<String>, // aux params in ON clause
}