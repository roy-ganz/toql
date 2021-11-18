use crate::{join_handler::JoinHandler, role_expr::RoleExpr, sql_arg::SqlArg};
use std::{collections::HashMap, sync::Arc};

/// Options for a mapped join.
#[derive(Debug)]
pub struct JoinOptions {
    pub(crate) key: bool, // Always select this join, regardless of query fields
    pub(crate) preselect: bool, // Always select this join, regardless of query fields
    pub(crate) partial_table: bool, // This joins to a table that shares the same primary key(s)
    pub(crate) skip_mut: bool, // Ignore field for updates
    pub(crate) load_role_expr: Option<RoleExpr>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Additional build params
    pub(crate) join_handler: Option<Arc<dyn JoinHandler + Send + Sync>>, // Optional join handler
}

impl JoinOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        JoinOptions {
            key: false,
            preselect: false,
            partial_table: false,
            skip_mut: false,
            load_role_expr: None,
            aux_params: HashMap::new(),
            join_handler: None,
        }
    }

    /// Mark join as a key.
    pub fn key(mut self, key: bool) -> Self {
        self.key = key;
        self
    }

    /// Use custom handler to build join.
    pub fn handler<H>(mut self, handler: H) -> Self
    where
        H: 'static + JoinHandler + Send + Sync,
    {
        self.join_handler = Some(Arc::new(handler));
        self
    }

    /// Mark join as preselected.
    /// The join must always be loaded, regardless what the [Query](crate::query::Query) selects.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    ///  Mark join as part of a partial table.
    pub fn partial_table(mut self, partial_table: bool) -> Self {
        self.partial_table = partial_table;
        self
    }

    /// Field is ignored by the wildcard.
    pub fn skip_mut(mut self, skip: bool) -> Self {
        self.skip_mut = skip;
        self
    }
    /// The field can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_load(mut self, role_expr: RoleExpr) -> Self {
        self.load_role_expr = Some(role_expr);
        self
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
}

impl Default for JoinOptions {
    fn default() -> Self {
        Self::new()
    }
}
