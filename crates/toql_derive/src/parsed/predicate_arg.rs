use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PredicateArg {
    /// SQL expression
    pub sql: String,
    //// Custom predicate handler, if any
    /// Stores the name of the function
    pub handler: Option<syn::Path>,
    /// Multiple aux params that can be used in ON clauses
    /// HashMap contains name, index of positional arguemnt in SQL expression
    pub on_aux_params: HashMap<String, usize>,
    /// Predicate should be used for count queries
    pub count_filter: bool,
}
