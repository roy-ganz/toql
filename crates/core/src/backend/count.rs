use super::{map, Backend};
use crate::toql_api::count::Count;
use crate::{
    alias_translator::AliasTranslator, error::ToqlError, parameter_map::ParameterMap, query::Query,
    sql_builder::SqlBuilder, table_mapper::mapped::Mapped,
};
use std::borrow::Borrow;

pub async fn count<B, Q, T, R, E>(backend: &mut B, query: Q) -> std::result::Result<u64, E>
where
    B: Backend<R, E>,
    Q: Borrow<Query<T>> + Send + Sync,
    T: Count,
    E: From<ToqlError>,
{
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    let ty = <T as Mapped>::type_name();
    let alias_format = backend.alias_format();
    let mut alias_translator = AliasTranslator::new(alias_format);
    let aux_params = [backend.aux_params()];
    let aux_params = ParameterMap::new(&aux_params);

    let sql = {
        let registry = &*backend.registry()?;
        let mut builder = SqlBuilder::new(&ty, registry)
            .with_aux_params(backend.aux_params().clone()) // TODO ref
            .with_roles(backend.roles().clone()); // TODO ref;
        let result = builder.build_count("", query.borrow(), false)?; // All filters, not just count selections

        result
            .to_sql(&aux_params, &mut alias_translator)
            .map_err(ToqlError::from)?
    };

    let page_count = backend.select_count_sql(sql).await?;
    Ok(page_count)
}
