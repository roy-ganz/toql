//! ### Registry
//! If a struct contains merged fields (collections of structs) then the SQL Builder must build multiple SQL queries with different mappers.
//! To give high level functions all SQL Mappers, they must be put into a registry. This allows to
//! load the full dependency tree.
//!

use crate::{
    field_handler::FieldHandler, result::Result, table_mapper::mapped::Mapped, table_mapper::TableMapper,
};
use heck::MixedCase;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TableMapperRegistry {
    pub mappers: HashMap<String, TableMapper>,
    //  pub alias_format: AliasFormat, //
}
impl TableMapperRegistry {
    pub fn new() -> TableMapperRegistry {
        TableMapperRegistry {
            mappers: HashMap::new(),
        }
    }
    pub fn get(&self, name: &str) -> Option<&TableMapper> {
        self.mappers.get(name)
    }

    /* pub fn with_alias_format(alias_format: AliasFormat) -> TableMapperRegistry {
        TableMapperRegistry {
            mappers: HashMap::new(),
      //      alias_format,
        }
    }  */
    pub fn insert(&mut self, mapper: TableMapper) {
        // Mixed case corresponds to toql path
        self.mappers
            .insert(mapper.table_name.to_mixed_case(), mapper);
    }
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> Result<String> {
        let m = TableMapper::from_mapped::<M>()?;
        self.mappers.insert(M::type_name(), m);
        tracing::event!(tracing::Level::INFO, ty = %M::type_name(), "Registered database schema for type.");
        Ok(M::type_name())
    }
    pub fn insert_new_mapper_with_handler<M: Mapped, H>(&mut self, handler: H) -> Result<String>
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let m = TableMapper::from_mapped_with_handler::<M, _>(handler)?;
        // m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(M::type_name(), m);
        Ok(M::type_name())
    }
}

impl Default for TableMapperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
