use std::collections::{HashMap, HashSet};
use crate::sql_arg::SqlArg;
use crate::join_handler::JoinHandler;
use std::sync::Arc;

/// Options for a mapped field.
#[derive(Debug)]
pub struct JoinOptions {
    pub(crate) preselect: bool, // Always select this join, regardless of query fields
    pub(crate) skip_wildcard: bool, // Ignore field on this join for wildcard selection
    pub(crate) roles: HashSet<String>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Additional build params
    pub(crate) join_handler: Option<Arc<dyn JoinHandler + Send + Sync>> // Optional join handler
        
}

impl JoinOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        JoinOptions {
            preselect: false,
            skip_wildcard: false,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            join_handler:None
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// Field is ignored by the wildcard.
    pub fn skip_wildcard(mut self, skip_wildcard: bool) -> Self {
        self.skip_wildcard = skip_wildcard;
        self
    }
    /// The field can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
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
}