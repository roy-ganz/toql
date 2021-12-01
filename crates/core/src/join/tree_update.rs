use super::Join;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::query::field_path::FieldPath;
use crate::table_mapper::mapped::Mapped;
use crate::tree::tree_update::TreeUpdate;

impl<T> TreeUpdate for Join<T>
where
    T: Keyed + TreeUpdate + Mapped,
    <T as Keyed>::Key: Key + Clone,
{
    fn update<'a, I>(
        &self,
        descendents: I,
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<crate::sql_expr::SqlExpr>,
    ) -> Result<(), crate::error::ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        match self {
            Join::Key(_) => Ok(()),
            Join::Entity(e) => e.update(descendents, fields, roles, exprs),
        }
    }
}
