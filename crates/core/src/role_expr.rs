pub enum RoleExpr {
    And(Box<RoleExpr>, Box<RoleExpr>),
    Or(Box<RoleExpr>, Box<RoleExpr>),
    Not(Box<RoleExpr>),
    Role(String),
    Invalid
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
    pub fn not(self) -> Self {
        RoleExpr::Not(Box::new(self))
    }

}