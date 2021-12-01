//! Cache to lookup static table information and SQL statements.

use crate::table_mapper_registry::TableMapperRegistry;
use std::{collections::HashSet, sync::RwLock};

/// Cache keeps static table information in the [TableMapperRegistry](crate::table_mapper_registry::TableMapperRegistry) and
/// may lookup SQL statements to bypass the [SqlBuilder](crate::sql_builder::SqlBuilder). However this
/// is currently not implemented.
pub struct Cache {
    pub registry: RwLock<TableMapperRegistry>,
    pub registered_roots: RwLock<HashSet<String>>,
}

impl Cache {
    /// Creates a new `Cache` with 200 cache entries.
    pub fn new() -> Self {
        Cache {
            registry: RwLock::new(TableMapperRegistry::default()),
            registered_roots: RwLock::new(HashSet::new()),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
