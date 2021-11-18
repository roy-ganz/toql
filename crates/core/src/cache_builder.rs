//! Create a cache to hold table schema and queries.
/// The capacity for the query cache can be chosen.
/// A query cache caüacity of 0 disables the query cache.
use crate::{cache::Cache, table_mapper_registry::TableMapperRegistry};
use lru::LruCache;
use std::{collections::HashSet, sync::RwLock};

pub struct CacheBuilder {
    capacity: usize,
}

impl CacheBuilder {
    pub fn new() -> Self {
        CacheBuilder { capacity: 50 }
    }

    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn into_cache(self) -> Cache {
        let registry = TableMapperRegistry::default();
        Cache {
            registry: RwLock::new(registry),
            registered_roots: RwLock::new(HashSet::new()),
            _query_cache: RwLock::new(LruCache::new(self.capacity)),
        }
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}
