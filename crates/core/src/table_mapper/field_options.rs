use crate::{role_expr::RoleExpr, sql_arg::SqlArg};
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// Options for a mapped field.
pub struct FieldOptions {
    pub(crate) preselect: bool, // Always select this field, regardless of query fields
    pub(crate) count_filter: bool, // Filter field on count query
    pub(crate) count_select: bool, // Select field on count query
    pub(crate) skip_mut: bool,  // Select field on mut select
    pub(crate) skip_wildcard: bool, // Skip field for wildcard selection
    pub(crate) skip_load: bool, // Select field for query builder
    pub(crate) load_role_expr: Option<RoleExpr>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Auxiliary params
    pub(crate) on_params: Vec<String>, // Identity params for on clauses
    pub(crate) key: bool,       // Field is part of key
}

impl FieldOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        FieldOptions {
            preselect: false,
            count_filter: false,
            count_select: false,
            skip_mut: false,
            skip_wildcard: false,
            skip_load: false,
            load_role_expr: None,
            aux_params: HashMap::new(),
            on_params: Vec::new(),
            key: false,
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// Field is part of key.
    /// It cannot be update and is always preselected
    pub fn key(mut self, key: bool) -> Self {
        self.key = key;
        self
    }

    /// Any filter on the field is considered when creating a count query.
    /// Typically applied to fields that represent permissions and foreign keys.
    /// Assumme a user wants to see all books. You will restrict the user query
    /// with a permission filter, so that the user sees all of *his* books.
    /// The count query must also use the filter.
    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }
    /// Any selected field is also used for the count query.
    /// Only used in rare cases where you fiddle with distinct results.
    pub fn count_select(mut self, count_select: bool) -> Self {
        self.count_select = count_select;
        self
    }
    /// Field is used for the mut select query.
    pub fn skip_mut(mut self, skip: bool) -> Self {
        self.skip_mut = skip;
        self
    }
    /// Field is used for the normal query.
    pub fn skip_load(mut self, skip: bool) -> Self {
        self.skip_load = skip;
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
