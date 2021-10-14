use super::Join;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::query::field_path::FieldPath;
use crate::tree::tree_update::TreeUpdate;
use crate::{error::ToqlError, table_mapper::mapped::Mapped};

impl<T> TreeUpdate for Join<T>
where
    T: Keyed + TreeUpdate + Mapped,
    <T as Keyed>::Key: Key + Clone,
{
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<crate::sql_expr::SqlExpr>,
    ) -> Result<(), crate::error::ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        match self {
            Join::Key(_k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                None => Ok(()), // Key has no columns to be updated
            },
            Join::Entity(e) => e.update(descendents, fields, roles, exprs),
        }
    }
}
