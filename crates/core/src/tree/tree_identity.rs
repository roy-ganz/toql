use crate::{error::ToqlError, query::field_path::FieldPath, sql_arg::SqlArg};
use std::{cell::RefCell, result::Result};

pub enum IdentityAction {
    Set(RefCell<Vec<SqlArg>>), // Needs interior mutability, because keys are taken from vec
    Refresh,
}
pub trait TreeIdentity {
    fn auto_id() -> bool;

    fn set_id<'a, 'b, I>(
        &mut self,
        descendents: &mut I,
        action: &'b IdentityAction,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
