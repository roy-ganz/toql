use crate::error::Result;
use crate::query::field_path::Descendents;
use crate::sql_arg::SqlArg;
use crate::sql_expr::SqlExpr;
pub trait TreePredicate {
    fn predicate<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        predicate: &mut SqlExpr,
        args: &mut Vec<SqlArg>,
    ) -> Result<()>;
}
