use super::{join_options::JoinOptions, join_type::JoinType};
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) joined_mapper: String,
    pub(crate) join_type: JoinType,
    pub(crate) table_expression: SqlExpr, // Table ...
    pub(crate) on_expression: SqlExpr,    // ON ...id = ..table_id
    pub(crate) options: JoinOptions,
}
