use crate::tree::tree_insert::TreeInsert;
use super::Join;
use crate::key::{Key, Keyed};
use crate::{sql_mapper::mapped::Mapped, error::ToqlError};
use crate::query::field_path::Descendents;
use crate::sql_expr::SqlExpr;
use std::collections::HashSet;


impl<T> TreeInsert for Join<T>
where T: Keyed + TreeInsert + Mapped, <T as Keyed>::Key: Key + Clone
{
    fn columns<'a>(
        descendents: &mut Descendents<'a>,  
    ) -> Result<SqlExpr, ToqlError> {
        <T as TreeInsert>::columns(descendents)
    }
    fn values<'a>(
        &self,
        descendents: &mut Descendents<'a>,
        roles: &HashSet<String>,
        values:  &mut SqlExpr  
   ) -> Result<(), ToqlError> {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(
                    p.as_str().to_string(),
                )),
                None => {
                    <<Self as Keyed>::Key as Key>::params(&k).into_iter()
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