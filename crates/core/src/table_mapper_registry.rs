//! A registry for all table mappers.

use crate::{
    field_handler::FieldHandler, result::Result, table_mapper::mapped::Mapped,
    table_mapper::TableMapper,
};
use heck::MixedCase;
use std::collections::HashMap;

/// The registry hold all table mappers togheter.
#[derive(Debug)]
pub struct TableMapperRegistry {
    pub mappers: HashMap<String, TableMapper>,
}
impl TableMapperRegistry {
    /// Create new registry.
    pub fn new() -> TableMapperRegistry {
        TableMapperRegistry {
            mappers: HashMap::new(),
        }
    }
    /// Get a mapper for a table name.
    pub fn get(&self, table_name: &str) -> Option<&TableMapper> {
        self.mappers.get(table_name)
    }

    /// Insert a mapper. Take the table name from the mnapper.
    pub fn insert(&mut self, mapper: TableMapper) {
        // Mixed case corresponds to toql path
        self.mappers
            .insert(mapper.table_name.to_mixed_case(), mapper);
    }
    /// Insert a mapper for a given struct.
    /// The [Mapped] trait is implemented for every Topql derived struct.
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> Result<String> {
        let m = TableMapper::from_mapped::<M>()?;
        self.mappers.insert(M::type_name(), m);
        tracing::event!(tracing::Level::INFO, ty = %M::type_name(), "Registered table mapping for type.");
        Ok(M::type_name())
    }
    /// Insert a mapper for a given struct with a [FieldHandler]
    /// This allows a custom field handler for a given struct.
    /// The [Mapped] trait is implemented for every Topql derived struct.
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
