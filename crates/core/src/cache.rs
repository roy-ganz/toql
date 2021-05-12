use crate::sql_mapper_registry::SqlMapperRegistry;
use lru::LruCache;
use std::{collections::HashSet, sync::RwLock};

pub struct Cache {
    pub registry: RwLock<SqlMapperRegistry>,
    pub registered_roots: RwLock<HashSet<String>>,
    pub(crate) _query_cache: RwLock<LruCache<String, String>>, // TODO Support cache lookup
}

impl Cache {
    pub fn with_capacity(capacity: usize) -> Self {
        Cache {
            registry: RwLock::new(SqlMapperRegistry::new()),
            registered_roots: RwLock::new(HashSet::new()),
            _query_cache: RwLock::new(LruCache::new(capacity)),
        }
    }

    pub fn new() -> Self {
        Self::with_capacity(50)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
