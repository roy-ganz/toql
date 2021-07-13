

use crate::{
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
   query::{Query}, sql_builder::{SqlBuilder}, alias_translator::AliasTranslator, parameter_map::ParameterMap, tree::tree_map::TreeMap, 
};
use std::{borrow::{Borrow},};
use super::{map, Backend};

pub trait Count: Keyed + Mapped + TreeMap + std::fmt::Debug {}

pub async fn count<B, Q, T, R, E>(backend: &mut B,query: Q) -> std::result::Result<u64, E>
where
    B: Backend<R, E>, 
    Q: Borrow<Query<T>> + Send + Sync,
    T: Count,
    E: From<ToqlError> 
  
 {
   
            {
                let registry = &mut *backend.registry_mut()?;
                map::map::<T>(registry)?;
            }

            let ty = <T as Mapped>::type_name();
            //let sql = build_load_count_sql(self.alias_format(), registry, ty)?;

            let alias_format = backend.alias_format();
            let mut alias_translator = AliasTranslator::new(alias_format);
            let aux_params = [backend.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);

            let sql ={
                let registry =  &*backend.registry()?;
                let mut builder = SqlBuilder::new(&ty, registry)
                    .with_aux_params(backend.aux_params().clone()) // todo ref
                    .with_roles(backend.roles().clone()); // todo ref;
                let result = builder.build_count("", query.borrow(), true)?;
                let sql = result
                    .to_sql(&aux_params, &mut alias_translator)
                    .map_err(ToqlError::from)?;
                sql
            };
        

            log_sql!(&sql);
            
            let page_count = backend.select_count_sql(sql).await?;
            
    
    Ok(page_count)
}
