use super::Join;
use crate::{
    error::ToqlError, key::Key, keyed::Keyed, query::field_path::FieldPath, sql_expr::SqlExpr,
    sql_mapper::mapped::Mapped, tree::tree_insert::TreeInsert,
};
use std::collections::HashSet;

impl<T> TreeInsert for Join<T>
where
    T: Keyed + TreeInsert + Mapped,
    <T as Keyed>::Key: Key + Clone,
{
    fn columns<'a, I>(descendents: &mut I) -> Result<SqlExpr, ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        <T as TreeInsert>::columns(descendents)
    }
    fn values<'a, I>(
        &self,
        descendents: &mut I,
        roles: &HashSet<String>,
        values: &mut SqlExpr,
    ) -> Result<(), ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                None => {
                    <<Self as Keyed>::Key as Key>::params(&k)
                        .into_iter()
                        .for_each(|a| {
                            values.push_arg(a);
                            values.push_literal(", ");
                        });
                    values.pop();
                    Ok(())
                }
            },
            Join::Entity(e) => e.values(descendents, roles, values),
        }
    }
}
