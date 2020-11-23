use crate::sql_mapper_registry::SqlMapperRegistry;
use std::collections::HashSet;
use lru::LruCache;

pub struct Cache {
    pub registry: SqlMapperRegistry,
    pub registered_roots: HashSet<String>,
    pub (crate) query_cache : LruCache<String, String>
}

impl Cache {

    pub fn with_capacity(capacity:usize) -> Self {

        Cache {
            registry: SqlMapperRegistry::new(),
            registered_roots : HashSet::new(),
            query_cache : LruCache::new(capacity)

        }
    }

     pub fn new() -> Self {
         Self::with_capacity(50)
    }


}