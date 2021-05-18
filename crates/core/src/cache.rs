//! Cache to lookup static table information and SQL statements.

use crate::sql_mapper_registry::SqlMapperRegistry;
use lru::LruCache;
use std::{collections::HashSet, sync::RwLock};

/// Cache keeps static table information in the [SQLMapperRegistry](crate::sql_mapper_registry::SqlMapperRegsitry) and 
/// may lookup SQL statements to bypass the [SqlBuilder](crate::sql_builder::SqlBuilder). However this 
/// is currently not implemented. 
pub struct Cache {
    pub registry: RwLock<SqlMapperRegistry>,
    pub registered_roots: RwLock<HashSet<String>>,
    pub(crate) _query_cache: RwLock<LruCache<String, String>>, // TODO Support cache lookup
}

impl Cache {
    /// Creates a new ``Cache` with `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Cache {
            registry: RwLock::new(SqlMapperRegistry::new()),
            registered_roots: RwLock::new(HashSet::new()),
            _query_cache: RwLock::new(LruCache::new(capacity)),
        }
    }
    /// Creates a new `Cache` with 50 cache entries.
    pub fn new() -> Self {
        Self::with_capacity(50)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
