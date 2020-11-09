use crate::error::ToqlError;
use crate::query::field_path::Descendents;
use crate::sql_expr::SqlExpr;
use crate::sql_arg::SqlArg;
use crate::sql_expr::PredicateColumn;


pub trait TreePredicate {

    fn columns<'a>(&self,descendents: &mut Descendents<'a> ) -> Result<Vec<String>, ToqlError>;
        
    fn args<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        args: &mut Vec<SqlArg>
    ) -> Result<(), ToqlError>;
}
