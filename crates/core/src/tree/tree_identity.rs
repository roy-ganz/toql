use crate::query::field_path::Descendents;
use crate::{error::ToqlError, sql_arg::SqlArg};
use std::{cell::RefCell, result::Result};


pub enum IdentityAction {
    Set(RefCell<Vec<SqlArg>>), // Needs interior mutability, because keys are taken from vec
    Refresh
}
pub trait TreeIdentity {

    fn auto_id() -> bool;

    fn set_id<'a, 'b>(
        &mut self,
        descendents: &mut Descendents<'a>,
        action: &'b IdentityAction,
    ) -> Result<(), ToqlError>;
}
