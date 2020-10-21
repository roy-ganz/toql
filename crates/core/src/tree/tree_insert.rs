use crate::{
    from_row::FromRow, query::field_path::Descendents, sql_builder::select_stream::SelectStream, sql_expr::SqlExpr, error::ToqlError,
};
use std::{collections::HashMap, ops::Index};
use crate::sql_arg::SqlArg;

// Trait is implemented for structs that can insert 
pub trait TreeInsert {
    fn columns<'a>(
        descendents: &mut Descendents<'a>,  
    ) -> Result<SqlExpr, ToqlError>;
     fn values<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        values:  &mut crate::sql_expr::SqlExpr  
   ) -> Result<(), ToqlError>; 
   /*  fn values<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
   ) -> Result<SqlExpr, ToqlError>; */
}
