use super::{map, Backend};
use crate::{
    alias_translator::AliasTranslator, error::ToqlError, parameter_map::ParameterMap, query::Query,
    sql_builder::SqlBuilder, sql_expr::SqlExpr, table_mapper::mapped::Mapped,
};
use std::borrow::Borrow;

use crate::toql_api::delete::Delete;

pub async fn delete<B, Q, T, R, E>(backend: &mut B, query: Q) -> std::result::Result<(), E>
where
    B: Backend<R, E>,
    T: Delete,
    Q: Borrow<Query<T>>,
    E: From<ToqlError>,
{
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    let mut result = SqlBuilder::new(
        &<T as Mapped>::type_name(),
        &*backend.registry().map_err(ToqlError::from)?,
    )
    .with_aux_params(backend.aux_params().clone()) // todo ref
    .with_roles(backend.roles().clone()) // todo ref
    .build_delete(query.borrow())?;

    // Add alias after Verb
    {
        let registry = &*backend.registry().map_err(ToqlError::from)?;
        let mapper = registry
            .get(&<T as Mapped>::type_name())
            .ok_or_else(|| ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        result.push_select(SqlExpr::alias(mapper.canonical_table_alias.to_owned()));
    }

    // No arguments, nothing to delete
    if result.is_empty() {
        Ok(())
    } else {
        let pa = [backend.aux_params()];
        let p = ParameterMap::new(&pa);
        let mut alias_translator = AliasTranslator::new(backend.alias_format());
        let sql = result
            .to_sql(&p, &mut alias_translator)
            .map_err(ToqlError::from)?;
        backend.execute_sql(sql).await
    }
}
