//! Trait to set key.
use crate::{error::ToqlError, query::field_path::FieldPath, sql_arg::SqlArg};
use std::{cell::RefCell, result::Result};

/// The action that [TreeIdentity] should do
pub enum IdentityAction {
    /// Set key to value (primary + dependencies).
    /// Argument needs interior mutability, because keys are taken from `Vec`.
    Set(RefCell<Vec<SqlArg>>), 
    /// Set only invalid keys to value  (primary + dependencies).
    /// Argument needs interior mutability, because keys are taken from `Vec`.
    SetInvalid(RefCell<Vec<SqlArg>>), 
    /// Refresh all foreign keys, that refer to this entity (merges).
    Refresh,
     /// Refresh all invalid foreign keys, that refer to this entity (merges).
    RefreshInvalid,
    /// Refresh all valid foreign keys, that refer to this entity (merges).
    RefreshValid,
}
/// Deal with primary and foreign keys in nested structs.
pub trait TreeIdentity {
    /// Returns true, if struct loacted at `descendents` has database generated keys.
    fn auto_id<'a, I>(descendents: I) -> Result<bool, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;

    /// Set or refresh keys of structs located at `descendents`.
    fn set_id<'a, 'b, I>(
        &mut self,
        descendents: I,
        action: &'b IdentityAction,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone;
}
