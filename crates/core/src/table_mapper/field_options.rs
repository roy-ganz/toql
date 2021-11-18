use crate::{field_handler::FieldHandler, role_expr::RoleExpr, sql_arg::SqlArg};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
/// Options for a mapped field.
pub struct FieldOptions {
    pub(crate) preselect: bool, // Always select this field, regardless of query fields
    pub(crate) skip_mut: bool,  // Select field on mut select
    pub(crate) skip_wildcard: bool, // Skip field for wildcard selection
    pub(crate) load_role_expr: Option<RoleExpr>, // Only for use by these roles
    pub(crate) update_role_expr: Option<RoleExpr>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Auxiliary params
    pub(crate) on_aux_params: Vec<String>, // Identity params for on clauses
    pub(crate) key: bool,       // Field is part of key
    pub(crate) field_handler: Option<Arc<dyn FieldHandler + Send + Sync>>, // Optional join handler
}

impl FieldOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        FieldOptions {
            preselect: false,
            skip_mut: false,
            skip_wildcard: false,
            load_role_expr: None,
            update_role_expr: None,
            aux_params: HashMap::new(),
            on_aux_params: Vec::new(),
            key: false,
            field_handler: None,
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// Use custom handler to build field.
    pub fn handler<H>(mut self, handler: H) -> Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        self.field_handler = Some(Arc::new(handler));
        self
    }

    /// Field is part of key.
    /// It cannot be update and is always preselected
    pub fn key(mut self, key: bool) -> Self {
        self.key = key;
        self
    }

    /// Field is used for the mut select query.
    pub fn skip_mut(mut self, skip: bool) -> Self {
        self.skip_mut = skip;
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
    pub fn restrict_load(mut self, role_expr: RoleExpr) -> Self {
        self.load_role_expr = Some(role_expr);
        self
    }
    /// The field can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_update(mut self, role_expr: RoleExpr) -> Self {
        self.update_role_expr = Some(role_expr);
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

impl Default for FieldOptions {
    fn default() -> Self {
        Self::new()
    }
}
