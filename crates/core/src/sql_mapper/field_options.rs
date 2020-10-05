use crate::sql_arg::SqlArg;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
/// Options for a mapped field.
pub struct FieldOptions {
    pub(crate) preselect: bool, // Always select this field, regardless of query fields
    pub(crate) count_filter: bool, // Filter field on count query
    pub(crate) count_select: bool, // Select field on count query
    pub(crate) mut_select: bool, // Select field on mut select
    pub(crate) skip_wildcard: bool, // Skip field for wildcard selection
    pub(crate) query_select: bool, // Select field for query builder
    pub(crate) roles: HashSet<String>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Auxiliary params
    pub(crate) on_params: Vec<String>, // Identity params for on clauses
}

impl FieldOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        FieldOptions {
            preselect: false,
            count_filter: false,
            count_select: false,
            mut_select: false,
            skip_wildcard: false,
            query_select: true,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            on_params: Vec::new(),
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
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
    pub fn mut_select(mut self, mut_select: bool) -> Self {
        self.mut_select = mut_select;
        self
    }
    /// Field is used for the normal query.
    pub fn query_select(mut self, query_select: bool) -> Self {
        self.query_select = query_select;
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
    where
        S: Into<String>,
        T: Into<SqlArg>,
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }
}
