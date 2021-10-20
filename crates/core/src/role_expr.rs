//! A boolean role expression that can be evaluated.

///  A `RoleExpr` is a boolean expression tree.
///
/// It is typically parsed by the [RoleExprParser](crate::role_expr_parser::RoleExprParser).
/// However it can also be build programmatically.
///
/// Here an example for the expression `admin or power_user`:
/// ```rust
/// use toql::prelude::RoleExpr;
///
/// let e = RoleExpr::Role("admin".to_string())
///                .or(RoleExpr::Role("power_user".to_string()));
/// assert_eq!("admin;power_user", e.to_string());
/// ```
/// To validate a role expression use the [RoleValidator](crate::role_validator::RoleValidator).
///
/// `RoleExpr` are used by Toql derive generated code.
///
/// End users should restrict actions with role expressions through the Toql derive.
///
/// ### Example
/// Restricting the field's selection to the roles 'admin' or 'power_user'
/// ```rust
/// #[derive(Toql)]
/// struct FooBar {
///   #[toql(key)]
///   id: u64,
///
///   #[toql(roles(load="admin;power_user"))]
///   name: Option<String>
/// }
/// ```
///
#[derive(Debug, Clone)]
pub enum RoleExpr {
    /// Concatenate both nodes with AND
    And(Box<RoleExpr>, Box<RoleExpr>),
    /// Concatenate both nodes with OR
    Or(Box<RoleExpr>, Box<RoleExpr>),
    /// Negate node
    Not(Box<RoleExpr>),
    /// This node is a role name
    Role(String),
    /// This node is always invalid
    Invalid,
}

impl RoleExpr {
    // Create a role expression that is always invalid
    pub fn invalid() -> Self {
        RoleExpr::Invalid
    }
    // Create a role for the given name
    pub fn role(role: String) -> Self {
        RoleExpr::Role(role)
    }
    // Concatenate this role and another role with AND
    pub fn and(self, role_expr: RoleExpr) -> Self {
        RoleExpr::And(Box::new(self), Box::new(role_expr))
    }
    // Concatenate this role and another role with OR
    pub fn or(self, role_expr: RoleExpr) -> Self {
        RoleExpr::Or(Box::new(self), Box::new(role_expr))
    }
    // Negate this role expression
    #[allow(clippy::clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        RoleExpr::Not(Box::new(self))
    }
}

impl ToString for RoleExpr {
    fn to_string(&self) -> String {
        match self {
            RoleExpr::And(a, b) => {
                format!("{}, {}", a.to_string(), b.to_string())
            }
            RoleExpr::Or(a, b) => {
                format!("({}); ({})", a.to_string(), b.to_string())
            }
            RoleExpr::Not(a) => a.to_string(),
            RoleExpr::Role(r) => r.to_string(),
            RoleExpr::Invalid => "`false`".to_string(),
        }
    }
}
