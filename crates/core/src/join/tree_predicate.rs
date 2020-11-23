use super::Join;
use crate::tree::tree_predicate::TreePredicate;
use crate::error::ToqlError;
use crate::query::field_path::Descendents;
use crate::key::{Key, Keyed};
use crate::sql_arg::SqlArg;


impl<T> TreePredicate for Join<T>
where
    T: Keyed,
    <T as Keyed>::Key: Clone,
    T: TreePredicate,
{
    fn columns<'a>(
        &self,
        descendents: &mut Descendents<'a>,
    ) -> Result<Vec<String>,ToqlError> {
        match self {
            Join::Key(_) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(
                    p.as_str().to_string(),
                )),
                None => Ok(<<Self as Keyed>::Key as Key>::columns()),
            },
            Join::Entity(e) => e.columns(descendents),
        }
    }
    fn args<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        args: &mut Vec<SqlArg>,
    ) -> Result<(),ToqlError> {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(
                    p.as_str().to_string(),
                )),
                None => {
                    args.extend(<<Self as Keyed>::Key as Key>::params(&k));
                    Ok(())
                }
            },
            Join::Entity(e) => e.args(descendents, args),
        }
    }
}