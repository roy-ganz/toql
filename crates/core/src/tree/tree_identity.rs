use crate::query::field_path::Descendents;
use crate::{error::ToqlError, sql_arg::SqlArg};
use std::result::Result;

#[derive(Clone)]
pub enum IdentityAction {
    Set(Vec<SqlArg>),
    Refresh
}
pub trait TreeIdentity {

    fn auto_id() -> bool;

    fn set_id<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
        action: IdentityAction,
    ) -> Result<(), ToqlError>;
}
