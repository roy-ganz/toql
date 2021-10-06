//! Check if a role expression is valid.
//
use crate::role_expr::RoleExpr;
use std::collections::HashSet;

pub struct RoleValidator {}

impl RoleValidator {
    pub fn is_valid(roles: &HashSet<String>, role_expr: &RoleExpr) -> bool {
        match role_expr {
            RoleExpr::Invalid => false,
            RoleExpr::And(a, b) => {
                Self::is_valid(roles, a.as_ref()) && Self::is_valid(roles, b.as_ref())
            }
            RoleExpr::Or(a, b) => {
                Self::is_valid(roles, a.as_ref()) || Self::is_valid(roles, b.as_ref())
            }
            RoleExpr::Not(a) => !Self::is_valid(roles, a.as_ref()),
            RoleExpr::Role(role) => roles.contains(role),
        }
    }
}
