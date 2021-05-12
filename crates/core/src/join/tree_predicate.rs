use super::Join;
use crate::error::ToqlError;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::query::field_path::{Descendents, FieldPath};
use crate::sql_arg::SqlArg;
use crate::tree::tree_predicate::TreePredicate;

impl<T> TreePredicate for Join<T>
where
    T: Keyed,
    <T as Keyed>::Key: Clone,
    T: TreePredicate,
{
    fn columns<'a, I>(&self, descendents: &mut I) -> Result<Vec<String>, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        match self {
            Join::Key(_) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                None => Ok(<<Self as Keyed>::Key as Key>::columns()),
            },
            Join::Entity(e) => e.columns(descendents),
        }
    }
    fn args<'a, I>(&self, descendents: &mut I, args: &mut Vec<SqlArg>) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                None => {
                    args.extend(<<Self as Keyed>::Key as Key>::params(&k));
                    Ok(())
                }
            },
            Join::Entity(e) => e.args(descendents, args),
        }
    }
}
