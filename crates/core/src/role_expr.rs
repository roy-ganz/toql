//! Keep a boolean role expression that can be evaluated to true or false for a given set of roles
/// The RoleExpr can also be used to build a role expression programmatically.
/// 
/// Here an example for the expression 'roles admin or power_user'
/// ```ignore
/// use toql::role_expr::RoleExpr;
/// let e = RoleExpr::Role("admin".to_string())
///                .and(RoleExpr::Role("power_user".to_string()));
/// assert_eq!("admin;power_user", e.to_string());
/// ```
/// To build a Role expr form a string, there is a [RoleExprParser](role_expr_parser/struct.RoleExprParser.html) to turn a string into a role expression.
/// Check there for the string notation of role expressions. 
///
/// To evaluate a role expression use the [RoleValidator](role_validator/struct.RoleValidator.html).
///
/// Note that this functionality should not be interesting for Toql *users*.
/// Role expressions are handled internally when requested through the struct mapping.
///
/// ## Example
/// Restricting the field's selection to the roles 'admin' or 'power_user' 
/// ```ignore
/// #[derive[Toql]]
/// struct FooBar {
///   #[toql(key)]
///   id: u64,
///   #[toql(roles(load="admin;power_user"))]
///   name: Option<String>
/// } 
/// ```




#[derive(Debug, Clone)]
pub enum RoleExpr {
    And(Box<RoleExpr>, Box<RoleExpr>),
    Or(Box<RoleExpr>, Box<RoleExpr>),
    Not(Box<RoleExpr>),
    Role(String),
    Invalid,
}

impl RoleExpr {
    pub fn invalid() -> Self {
        RoleExpr::Invalid
    }
    pub fn role(role: String) -> Self {
        RoleExpr::Role(role)
    }

    pub fn and(self, role_expr: RoleExpr) -> Self {
        RoleExpr::And(Box::new(self), Box::new(role_expr))
    }
    pub fn or(self, role_expr: RoleExpr) -> Self {
        RoleExpr::Or(Box::new(self), Box::new(role_expr))
    }
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
