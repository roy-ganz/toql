
use std::collections::{HashMap, HashSet};
use crate::sql::SqlArg;

#[derive(Debug)]
pub struct PredicateOptions {
     pub(crate) aux_params: HashMap<String, SqlArg>,
     pub(crate) on_params: Vec<(u8,String)>,  // Argument params for on clauses (index, name)
     pub(crate) count_filter: bool,
     pub(crate) roles: HashSet<String>, // Only for use by these roles
}

impl PredicateOptions {

    pub fn new() -> Self {
        PredicateOptions { aux_params: HashMap::new(), on_params: Vec::new(), count_filter: false, roles: HashSet::new()}
    }

 /// Additional build param. This is used by the query builder together with
     /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
     /// and field handlers.
    pub fn aux_param<S, T>(mut self, name: S, value: T) -> Self 
    where S: Into<String>, T:Into<SqlArg>
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }

    /// Additional build param. This is used by the query builder together with
     /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
     /// and field handlers.
    pub fn on_param(mut self, index: u8, name: String) -> Self {
        self.on_params.push((index, name));
        self
    }
    /// By default predicates are considered when creating a count query.
    /// However the predicate can be ignored by setting the count filter to false
    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }
}
