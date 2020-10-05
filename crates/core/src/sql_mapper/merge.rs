use crate::sql_expr::SqlExpr;

#[derive(Debug)]
pub(crate) struct Merge {
    pub(crate) merged_mapper: String,
    pub(crate) merge_join: SqlExpr,      // JOIN ..
    pub(crate) merge_predicate: SqlExpr, // ON ..
}
