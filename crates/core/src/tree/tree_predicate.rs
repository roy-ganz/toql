use crate::query::field_path::Descendents;
use crate::error::Result;
use crate::sql::Sql;
pub trait TreePredicate
{
    fn predicate<'a>(&self,  descendents: &Descendents<'a>, predicate: &mut Sql) -> Result<()>;
}