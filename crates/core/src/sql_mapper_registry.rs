//! ### Registry
//! If a struct contains merged fields (collections of structs) then the SQL Builder must build multiple SQL queries with different mappers.
//! To give high level functions all SQL Mappers, they must be put into a registry. This allows to
//! load the full dependency tree.
//!
/// A registry that holds mappers.
use crate::query::field_path::FieldPath;
use crate::alias::AliasFormat;
use crate::sql_mapper::{SqlMapper};
use crate::sql_mapper::mapped::Mapped;
use crate::field_handler::FieldHandler;
use std::collections::HashMap;
use heck::MixedCase;

#[derive(Debug)]
pub struct SqlMapperRegistry {
    pub mappers: HashMap<String, SqlMapper>,
    pub alias_format: AliasFormat, //
}
impl SqlMapperRegistry {
    pub fn new() -> SqlMapperRegistry {
        Self::with_alias_format(AliasFormat::Canonical)
    }
    pub fn get(&self, name: &str) -> Option<&SqlMapper> {
        self.mappers.get(name)
    }
    
    pub fn with_alias_format(alias_format: AliasFormat) -> SqlMapperRegistry {
        SqlMapperRegistry {
            mappers: HashMap::new(),
            alias_format,
        }
    }
    pub fn insert(&mut self, mapper: SqlMapper) 
    {
        // Mixed case corresponds to toql path
        self.mappers.insert(mapper.table_name.to_mixed_case(), mapper);
    }
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> String {
      let now = std::time::Instant::now();
        let m = SqlMapper::from_mapped::<M>();
        self.mappers.insert(String::from(M::type_name()), m);
         println!("Mapped `{}` in {}ms",  M::type_name(), now.elapsed().as_millis());
        M::type_name()
    }
    pub fn insert_new_mapper_with_handler<M: Mapped, H>(&mut self, handler: H) -> String
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let m = SqlMapper::from_mapped_with_handler::<M, _>(handler);
        // m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(String::from(M::type_name()), m);
        M::type_name()
    }
}
