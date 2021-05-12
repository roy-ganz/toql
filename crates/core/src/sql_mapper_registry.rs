//! ### Registry
//! If a struct contains merged fields (collections of structs) then the SQL Builder must build multiple SQL queries with different mappers.
//! To give high level functions all SQL Mappers, they must be put into a registry. This allows to
//! load the full dependency tree.
//!

use crate::{field_handler::FieldHandler, result::Result, sql_mapper::mapped::Mapped, sql_mapper::SqlMapper};
use heck::MixedCase;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SqlMapperRegistry {
    pub mappers: HashMap<String, SqlMapper>,
    //  pub alias_format: AliasFormat, //
}
impl SqlMapperRegistry {
    pub fn new() -> SqlMapperRegistry {
        SqlMapperRegistry {
            mappers: HashMap::new(),
        }
    }
    pub fn get(&self, name: &str) -> Option<&SqlMapper> {
        self.mappers.get(name)
    }

    /* pub fn with_alias_format(alias_format: AliasFormat) -> SqlMapperRegistry {
        SqlMapperRegistry {
            mappers: HashMap::new(),
      //      alias_format,
        }
    }  */
    pub fn insert(&mut self, mapper: SqlMapper) {
        // Mixed case corresponds to toql path
        self.mappers
            .insert(mapper.table_name.to_mixed_case(), mapper);
    }
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> Result<String> {
        let m = SqlMapper::from_mapped::<M>()?;
        self.mappers.insert(String::from(M::type_name()), m);
        log::info!("Mapped `{}`", M::type_name());
        Ok(M::type_name())
    }
    pub fn insert_new_mapper_with_handler<M: Mapped, H>(&mut self, handler: H) -> Result<String>
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let m = SqlMapper::from_mapped_with_handler::<M, _>(handler)?;
        // m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(String::from(M::type_name()), m);
        Ok(M::type_name())
    }
}
