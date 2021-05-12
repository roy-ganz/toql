use crate::alias::AliasFormat;
use crate::cache::Cache;
use crate::sql_mapper_registry::SqlMapperRegistry;
use lru::LruCache;
use std::{collections::HashSet, sync::RwLock};

pub struct CacheBuilder {
    capacity: usize,
    alias_format: AliasFormat,
}

impl CacheBuilder {
    pub fn new() -> Self {
        CacheBuilder {
            capacity: 50,
            alias_format: AliasFormat::Canonical,
        }
    }

    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }
    pub fn with_alias_format(mut self, alias_format: AliasFormat) -> Self {
        self.alias_format = alias_format;
        self
    }
    pub fn into_cache(self) -> Cache {
        let registry = SqlMapperRegistry::new();
        Cache {
            registry: RwLock::new(registry),
            registered_roots: RwLock::new(HashSet::new()),
            _query_cache: RwLock::new(LruCache::new(self.capacity)),
        }
    }
}
