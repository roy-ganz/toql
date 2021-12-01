use super::Join;
use crate::error::ToqlError;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::query::field_path::FieldPath;
use crate::sql_arg::SqlArg;
use crate::tree::tree_identity::{IdentityAction, TreeIdentity};
use std::convert::TryFrom;

impl<T> TreeIdentity for Join<T>
where
    T: Keyed,
    <T as Keyed>::Key: Clone,
    T: TreeIdentity,
    <T as Keyed>::Key: TryFrom<Vec<SqlArg>, Error = ToqlError>,
{
    fn auto_id<'a, I>(descendents: I) -> Result<bool, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        <T as TreeIdentity>::auto_id(descendents)
    }
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b IdentityAction,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(_) => Ok(()),
                None => match action {
                    IdentityAction::Set(ids) => {
                        let n = <<T as Keyed>::Key as Key>::columns().len();
                        let end = ids.borrow().len();
                        let args: Vec<SqlArg> =
                            ids.borrow_mut().drain(end - n..).collect::<Vec<_>>();
                        let key = TryFrom::try_from(args)?;
                        *k = key;
                        Ok(())
                    }
                    IdentityAction::SetInvalid(ids) => {
                        let params = <<T as Keyed>::Key as Key>::params(k);
                        if crate::sql_arg::valid_key(&params) {
                            return Ok(());
                        }
                        let n = <<T as Keyed>::Key as Key>::columns().len();
                        let end = ids.borrow().len();
                        let args: Vec<SqlArg> =
                            ids.borrow_mut().drain(end - n..).collect::<Vec<_>>();
                        let key = TryFrom::try_from(args)?;
                        *k = key;
                        Ok(())
                    }
                    IdentityAction::Refresh => Ok(()),
                    IdentityAction::RefreshValid => Ok(()),
                    IdentityAction::RefreshInvalid => Ok(()),
                },
            },
            Join::Entity(e) => e.set_id(descendents, action),
        }
    }
}
