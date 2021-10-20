//! Validate a [RoleExpr](crate::role_expr::RoleExpr).

use crate::role_expr::RoleExpr;
use std::collections::HashSet;

/// The `RoleValidator` validates a role expression with a set of role names.
///
/// Roles are typically assigned to a user during an authentication process.
///
/// ### Example
/// Let's say a user authenticates and is given the roles _user_, _subscribed_ and _student_.
///
/// For those the role expression `user, teacher` is invalid
/// while the expression `user,teacher; subscribed` is valid.
pub struct RoleValidator {}

impl RoleValidator {
    /// Return true, if role expression is valid.
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
