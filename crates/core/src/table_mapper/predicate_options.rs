use crate::{predicate_handler::PredicateHandler, role_expr::RoleExpr, sql_arg::SqlArg};
use std::{collections::HashMap, sync::Arc};

/// Options for a mapped predicate.
#[derive(Debug)]
pub struct PredicateOptions {
    pub(crate) aux_params: HashMap<String, SqlArg>,
    pub(crate) on_aux_params: Vec<(usize, String)>, // Argument params for on clauses (index, name)
    pub(crate) count_filter: bool,
    pub(crate) load_role_expr: Option<RoleExpr>, // Only for use by these roles
    pub(crate) predicate_handler: Option<Arc<dyn PredicateHandler + Send + Sync>>, // Optional join handler
}

impl PredicateOptions {
    pub fn new() -> Self {
        PredicateOptions {
            aux_params: HashMap::new(),
            on_aux_params: Vec::new(),
            count_filter: false,
            load_role_expr: None,
            predicate_handler: None,
        }
    }

    /// Additional build param. This is used by the query builder together with
    /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
    /// and field handlers.
    pub fn aux_param<S, T>(mut self, name: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<SqlArg>,
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }

    /// Additional aux build param. This is used by the query builder together with
    /// the aux params from the [Query](crate::query::Query) or [Context](crate::backend::context::Context).
    /// Aux params can be used in SQL expressions like `SELECT <param_name>` and field handlers.
    pub fn on_aux_param(mut self, index: usize, name: String) -> Self {
        self.on_aux_params.push((index, name));
        self
    }
    /// By default predicates are _NOT_ considered when creating a count query.
    /// However the predicate can be included by setting the count filter to `true`.
    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }

    /// Use custom handler to build predicate.
    pub fn handler<H>(mut self, handler: H) -> Self
    where
        H: 'static + PredicateHandler + Send + Sync,
    {
        self.predicate_handler = Some(Arc::new(handler));
        self
    }
}

impl Default for PredicateOptions {
    fn default() -> Self {
        Self::new()
    }
}
