use crate::query::field_path::Descendents;
use crate::error::Result;
use crate::sql_expr::SqlExpr;
use crate::sql_arg::SqlArg;
pub trait TreePredicate
{
    fn predicate<'a>(&self,  descendents: &mut Descendents<'a>, predicate: &mut SqlExpr, args: &mut Vec<SqlArg>) -> Result<()>;
}