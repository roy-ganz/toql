
use crate::query::Query;
use crate::sql_mapper::SqlMapperCache;

use std::result::Result;

/// Page start, offset
pub enum Page {
    Uncounted(u64, u16),
    Counted(u64,u16)
}

/// Trait to load entities from database.
pub trait Load<T> {

    type error;
    /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn load_one(
        &mut self,
        query: &Query,
        mappers: &SqlMapperCache,
    ) -> Result<T, Self::error>;

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    fn load_many(
        &mut self,
        query: &Query,
        mappers: &SqlMapperCache,
        page: Page
    ) -> Result<(Vec<T>, Option<(u32, u32)>), Self::error>;


     fn load_path(
         &mut self,
        path: &str,
        query: &crate::query::Query,
        cache: &crate::sql_mapper::SqlMapperCache,
    ) -> Result<Option<std::vec::Vec<T>>, Self::error>;

     fn load_dependencies(
        &mut self,
        entities: &mut Vec<T>,
        query: &crate::query::Query,
        cache: &crate::sql_mapper::SqlMapperCache,
    ) -> Result<(), Self::error>;

}


