//! Cache to lookup static table information and SQL statements.

use crate::table_mapper_registry::TableMapperRegistry;
use lru::LruCache;
use std::{collections::HashSet, sync::RwLock};

/// Cache keeps static table information in the [TableMapperRegistry](crate::table_mapper_registry::TableMapperRegsitry) and 
/// may lookup SQL statements to bypass the [SqlBuilder](crate::sql_builder::SqlBuilder). However this 
/// is currently not implemented. 
pub struct Cache {
    pub registry: RwLock<TableMapperRegistry>,
    pub registered_roots: RwLock<HashSet<String>>,
    pub(crate) _query_cache: RwLock<LruCache<String, String>>, // TODO Support cache lookup
}

impl Cache {
    /// Creates a new ``Cache` with `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Cache {
            registry: RwLock::new(TableMapperRegistry::new()),
            registered_roots: RwLock::new(HashSet::new()),
            _query_cache: RwLock::new(LruCache::new(capacity)),
        }
    }
    /// Creates a new `Cache` with 200 cache entries.
    pub fn new() -> Self {
        Self::with_capacity(200)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
