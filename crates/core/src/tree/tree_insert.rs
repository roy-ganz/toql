use crate::{
   query::field_path::Descendents, sql_expr::SqlExpr, error::ToqlError,
};

// Trait is implemented for structs that can insert 
pub trait TreeInsert {
    fn columns<'a>(
        descendents: &mut Descendents<'a>,  
    ) -> Result<SqlExpr, ToqlError>;
     fn values<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        roles: &std::collections::HashSet<String>,
        values:  &mut crate::sql_expr::SqlExpr  
   ) -> Result<(), ToqlError>; 
  
}
