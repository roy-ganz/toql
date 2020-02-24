//! ### Registry
//! If a struct contains merged fields (collections of structs) then the SQL Builder must build multiple SQL queries with different mappers.
//! To give high level functions all SQL Mappers, they must be put into a registry. This allows to
//! load the full dependency tree.
//!
/// A registry that holds mappers.
use crate::alias::AliasFormat;
use crate::sql_mapper::{SqlMapper, Mapped};
use crate::field_handler::FieldHandler;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SqlMapperRegistry {
    pub mappers: HashMap<String, SqlMapper>,
    pub alias_format: AliasFormat, //
}
impl SqlMapperRegistry {
    pub fn new() -> SqlMapperRegistry {
        Self::with_alias_format(AliasFormat::Canonical)
    }
    pub fn with_alias_format(alias_format: AliasFormat) -> SqlMapperRegistry {
        SqlMapperRegistry {
            mappers: HashMap::new(),
            alias_format,
        }
    }
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> String {
        let m = SqlMapper::from_mapped_with_alias::<M>(&M::table_alias(), self.alias_format.clone());
        self.mappers.insert(String::from(M::type_name()), m);
        M::type_name()
    }
    pub fn insert_new_mapper_with_handler<M: Mapped, H>(&mut self, handler: H) -> String
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let m = SqlMapper::from_mapped_with_handler::<M, _>(self.alias_format.clone(), handler);
        // m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(String::from(M::type_name()), m);
        M::type_name()
    }
}
