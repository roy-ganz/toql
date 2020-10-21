use crate::query::field_path::Descendents;
use crate::{error::ToqlError};
use std::result::Result;

pub trait TreeIdentity {

    fn auto_id() -> bool;

    fn set_id<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
        id: u64,
    ) -> Result<(), ToqlError>;
}
