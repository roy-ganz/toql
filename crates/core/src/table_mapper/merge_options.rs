use crate::role_expr::RoleExpr;

/// Options for a mapped field.
#[derive(Debug)]
pub struct MergeOptions {
    pub(crate) preselect: bool, // Always select this merge, regardless of query fields
    //pub(crate) skip_mut: bool, // Ignore merge for updates
    pub(crate) load_role_expr: Option<RoleExpr>, // Only for use by these roles
}

impl MergeOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        MergeOptions {
            preselect: false,
            load_role_expr: None,
        }
    }

    /// Merge is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// The merge can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_load(mut self, role_expr: RoleExpr) -> Self {
        self.load_role_expr = Some(role_expr);
        self
    }
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self::new()
    }
}
