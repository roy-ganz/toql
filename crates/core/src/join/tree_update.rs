use super::Join;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::tree::tree_update::TreeUpdate;
use crate::{error::ToqlError, table_mapper::mapped::Mapped};
use crate::{query::field_path::FieldPath, sql_expr::resolver::Resolver, sql_arg::SqlArg};

impl<T> TreeUpdate for Join<T>
where
    T: Keyed + TreeUpdate + Mapped,
    <T as Keyed>::Key: Key + Clone,
{
    fn update<'a, I>(
        &self,
        descendents: &mut I,
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        selected_keys: Option<&[Vec<SqlArg>]>,
        exprs: &mut Vec<crate::sql_expr::SqlExpr>,
    ) -> Result<(), crate::error::ToqlError>
    where
        I: Iterator<Item = FieldPath<'a>>,
    {
        match self {
            Join::Key(k) => match descendents.next() {
                Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                None => {
                    let table_alias = <T as Mapped>::table_alias();
                    let resolver = Resolver::new().with_self_alias(&table_alias);
                    exprs.push(resolver.resolve(&k.predicate_expr(false))?);
                    Ok(())
                }
            },
            Join::Entity(e) => e.update(descendents, fields, roles, selected_keys, exprs),
        }
    }
}
