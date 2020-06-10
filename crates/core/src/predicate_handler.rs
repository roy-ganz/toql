

use std::collections::HashMap;
 use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::sql::{Sql, SqlArg};

pub trait PredicateHandler {
    
    
    /// Match filter and return SQL expression or None, if no filtering is required.
    /// Do not insert parameters in the SQL expression, use `?` instead and provide the argument in the vector.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_predicate(
        &self,
        expression: Sql,
        predicate_args: &Vec<SqlArg>,
        aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Option<Sql>, SqlBuilderError>;
   
    
}


impl std::fmt::Debug for (dyn PredicateHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PredicateHandler()")
    }
}

pub struct DefaultPredicateHandler;

impl DefaultPredicateHandler {

    pub fn new() -> Self {
        DefaultPredicateHandler {}
    }
}

impl PredicateHandler for DefaultPredicateHandler {

 fn build_predicate(
        &self,
        predicate: Sql,
        _predicate_args: &Vec<SqlArg>,
        _aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Option<Sql>, crate::sql_builder::sql_builder_error::SqlBuilderError> {
       // Wrap in parens
        Ok(Some(Sql(format!("({})", predicate.0),predicate.1)))
    }
   

}
