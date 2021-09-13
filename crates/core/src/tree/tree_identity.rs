use crate::{error::ToqlError, query::field_path::FieldPath, sql_arg::SqlArg};
use std::{cell::RefCell, result::Result};

pub enum IdentityAction {
    Set(RefCell<Vec<SqlArg>>), // Needs interior mutability, because keys are taken from vec
    Refresh,
    RefreshInvalid,
    RefreshValid,
}
pub trait TreeIdentity {
    fn auto_id<'a, I>( descendents: &mut I) -> Result<bool, ToqlError>
      where
        I: Iterator<Item = FieldPath<'a>>
    ;

    fn set_id<'a, 'b, I>(
        &mut self,
        descendents: &mut I,
        action: &'b IdentityAction,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
