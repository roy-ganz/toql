use super::field_options::FieldOptions;
use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) options: FieldOptions, // Options
    pub(crate) expression: SqlExpr,   // Column name or SQL expression
}
