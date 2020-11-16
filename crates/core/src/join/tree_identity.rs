use super::Join;
use crate::key::Keyed;
use crate::tree::tree_identity::{TreeIdentity, IdentityAction};
use crate::error::ToqlError;
use std::convert::TryFrom;
use crate::sql_arg::SqlArg;
use crate::query::field_path::Descendents;

impl<E> TreeIdentity for Join<E>
where
    E: Keyed,
    <E as Keyed>::Key: Clone,
    E: TreeIdentity,
    <E as Keyed>::Key: TryFrom<Vec<SqlArg>, Error=ToqlError >
{
    fn auto_id() -> bool {
        <E as TreeIdentity>::auto_id()
    }
    fn set_id<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
        action: IdentityAction,
    ) -> Result<(), ToqlError> {
       
       match self {
           Join::Key(k) => { 
               match descendents.next() {
               Some(p) => {  Err(ToqlError::ValueMissing(  p.as_str().to_string()))},
               None => {
                   match action {
                       IdentityAction::Set(args) => {
                            let key = TryFrom::try_from(args)?;
                            *k = key;
                            Ok(())
                       }
                       IdentityAction::Refresh => {Ok(())}
                   }
                   
               }
           }
            }
           Join::Entity(e) => { e.set_id(descendents, action)}
       }
       
    }
}