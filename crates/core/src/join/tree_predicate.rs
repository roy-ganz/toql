use super::Join;
use crate::error::ToqlError;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::{query::field_path::FieldPath, sql_arg::SqlArg, tree::tree_predicate::TreePredicate};

impl<T> TreePredicate for Join<T>
where
    T: Keyed,
    <T as Keyed>::Key: Clone,
    T: TreePredicate,
{
    fn columns<'a, I>(descendents: I) -> Result<Vec<String>, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        <T as TreePredicate>::columns(descendents)
    }
    fn args<'a, I>(&self, mut descendents: I, args: &mut Vec<SqlArg>) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(_) => Ok(()),
                None => {
                    args.extend(<<Self as Keyed>::Key as Key>::params(&k));
                    Ok(())
                }
            },
            Join::Entity(e) => e.args(descendents, args),
        }
    }
}
