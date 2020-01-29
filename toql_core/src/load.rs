use crate::query::Query;
use crate::sql_mapper::SqlMapperCache;

use std::result::Result;

/// Page start, offset
pub enum Page {
    Uncounted(u64, u16),
    Counted(u64, u16),
}

/// Trait to load entities from database.
/// This is implemented for each SQL dialect, whereas <T> is the entity to load: E.g. impl Load<User> for MySql
pub trait Load<T: crate::key::Key> {
    type error;
    /// Load a struct with dependencies for a given Toql query.
    ///
    /// /// Roles a query has to access fields.
    /// See [MapperOption](../sql_mapper/struct.MapperOptions.html#method.restrict_roles) for explanation.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn load_one(&mut self, query: &Query, mappers: &SqlMapperCache) -> Result<T, Self::error>;

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    fn load_many(
        &mut self,
        query: &Query,
        mappers: &SqlMapperCache,
        page: Page,
    ) -> Result<(Vec<T>, Option<(u32, u32)>), Self::error>;

    /*  fn load_path(
         &mut self,
        path: &str,
        query: &crate::query::Query,
        cache: &crate::sql_mapper::SqlMapperCache,
    ) -> Result<Option<std::vec::Vec<T>>, Self::error>; */

    /* fn load_path_with_keys<J, K>(
        &mut self,
        path: &str,
        query: &crate::query::Query,
        cache: &crate::sql_mapper::SqlMapperCache,

    ) -> Result<Option<std::vec::Vec<(T, J, K)>>, Self::error>

    {
        Ok(None)
    } */
    fn build_path(
        &mut self,
        path: &str,
        query: &crate::query::Query,
        cache: &crate::sql_mapper::SqlMapperCache,
    ) -> Result<crate::sql_builder_result::SqlBuilderResult, Self::error>;

    fn load_dependencies(
        &mut self,
        _entities: &mut Vec<T>,
        _entity_keys: &Vec<T::Key>,
        _query: &crate::query::Query,
        _cache: &crate::sql_mapper::SqlMapperCache,
    ) -> Result<(), Self::error> {
        Ok(())
    }
}
