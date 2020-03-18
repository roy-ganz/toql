

use std::collections::HashMap;
 use crate::sql_builder::SqlBuilderError;

pub trait PredicateHandler {
    
    
    /// Match filter and return SQL expression or None, if no filtering is required.
    /// Do not insert parameters in the SQL expression, use `?` instead and provide the argument in the vector.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_predicate(
        &self,
        expression: (String, Vec<String>),
        aux_params: &HashMap<String, String>,
    ) -> Result<Option<(String, Vec<String>)>, SqlBuilderError>;
   
    
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
        predicate: (String, Vec<String>),
        _aux_params: &HashMap<String, String>,
    ) -> Result<Option<(String, Vec<String>)>, crate::sql_builder::SqlBuilderError> {
        let mut sql_params= Vec::new();
        for p in predicate.1 {
            sql_params.push(p);
        }
        Ok(Some((format!("({})", predicate.0),sql_params)))
    }
   

}
