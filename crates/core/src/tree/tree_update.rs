use crate::{
   query::field_path::Descendents,  sql_expr::SqlExpr, error::ToqlError,
};

// Trait is implemented for structs that can update 
pub trait TreeUpdate {
    fn update<'a>(
        &self,
        descendents: &mut Descendents<'a>, 
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        exprs : &mut Vec<SqlExpr> 
    ) -> Result<(), ToqlError>;
     
  
}
