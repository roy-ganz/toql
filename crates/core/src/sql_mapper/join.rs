use super::join_options::JoinOptions;
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) joined_mapper: String,
    pub(crate) join_expression: SqlExpr, // INNER JOIN Table ...
    pub(crate) on_expression: SqlExpr,   // ON ...id = ..table_id
    pub(crate) options: JoinOptions,
}
