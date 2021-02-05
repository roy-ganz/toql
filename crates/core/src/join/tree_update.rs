use crate::tree::tree_update::TreeUpdate;
use super::Join;
use crate::key::Key;
use crate::keyed::Keyed;
use crate::{sql_mapper::mapped::Mapped, error::ToqlError};
use crate::sql_expr::resolver::Resolver;

impl<T> TreeUpdate for Join<T>
where T: Keyed + TreeUpdate + Mapped, <T as Keyed>::Key: Key + Clone
{
    fn update<'a>(
        &self,
        descendents: &mut crate::query::field_path::Descendents<'a>, 
        fields: &std::collections::HashSet<String>, // if empty, all fields can be updated (*)
        roles: &std::collections::HashSet<String>,
        exprs : &mut Vec<crate::sql_expr::SqlExpr> 
    ) -> Result<(), crate::error::ToqlError> {
        match self {
            Join::Key(k) => {
                match descendents.next() {
                    Some(p) => Err(ToqlError::ValueMissing(p.as_str().to_string())),
                    None => {
                        let table_alias = <T as Mapped>::table_alias();
                        let resolver = Resolver::new().with_self_alias(&table_alias);
                        exprs.push(resolver.resolve(&crate::key::predicate_expr(k.clone()))?);
                        Ok(())
                    }
                }
            }
            Join::Entity(e) => {
                e.update(descendents, fields, roles, exprs)
            }
        }
    }


}