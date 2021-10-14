use crate::{join_handler::JoinHandler, role_expr::RoleExpr, sql_arg::SqlArg};
use std::{collections::HashMap, sync::Arc};

/// Options for a mapped field.
#[derive(Debug)]
pub struct JoinOptions {
    pub(crate) key: bool, // Always select this join, regardless of query fields
    pub(crate) preselect: bool, // Always select this join, regardless of query fields
    pub(crate) partial_table: bool, // This joins to a table that shares the same primary key(s)
    pub(crate) skip_wildcard: bool, // Ignore field on this join for wildcard selection
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
            skip_wildcard: false,
            skip_mut: false,
            load_role_expr: None,
            aux_params: HashMap::new(),
            join_handler: None,
        }
    }

    /// Join is a key.
    pub fn key(mut self, key: bool) -> Self {
        self.key = key;
        self
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// Field is selected, regardless of the query.
    pub fn partial_table(mut self, partial_table: bool) -> Self {
        self.partial_table = partial_table;
        self
    }

    /// Field is ignored by the wildcard.
    pub fn skip_wildcard(mut self, skip_wildcard: bool) -> Self {
        self.skip_wildcard = skip_wildcard;
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
