use crate::query::Query;
use crate::sql_mapper_registry::SqlMapperRegistry;
use crate::sql_builder::WildcardScope;
use std::result::Result;

/// [Page](enum.Page.html) is used as an argument in load functions. It tells Toql to build and run an additional query
/// to count the total number of records. 
/// 
/// In Toql there are 2 types of counts: A filtered count and a total count. Lets take a datagrid where the user searches his contacts with a name starting with 'Alice'.
/// The datagrid would show the following:
///  - Total number of contacts (Total count)
///  - Number of found contacts with the name 'Alice' (Filtered count)
///
/// While the filtered count is almost for free and returned for every query, 
/// the total count needs a seperate query with a different SQL filter predicate. 
/// Toql can do that out of the box, but the fields must be mapped accordingly in the [SqlMapper](../sql_mapper/struct.SqlMapper.html)
pub enum Page {
    /// Retrieve filtered count only.
    /// Argments are *start index* and *number of records*.
    Uncounted(u64, u16),
    // Retrieve filtered count and total count.
    /// Argments are *start index* and *number of records*.
    Counted(u64, u16),
}

/// Trait to load entities from database.
/// This is implemented for each struct in each SQL dialect. E.g. `impl Load<User> for MySql<..>`
pub trait Load<T: crate::key::Key> {
    type Error;
    /// Load a struct with dependencies for a given Toql query.
    ///
    /// /// Roles a query has to access fields.
    /// See [MapperOption](../sql_mapper/struct.MapperOptions.html#method.restrict_roles) for explanation.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn load_one(&mut self, query: &Query, mappers: &SqlMapperRegistry) -> Result<T, Self::Error>;

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    fn load_many(
        &mut self,
        query: &Query,
        mappers: &SqlMapperRegistry,
        page: Page,
    ) -> Result<(Vec<T>, Option<(u32, u32)>), Self::Error>;

    /// Build SQL for a toql path. This is used by the Toql derive to load a collection (merged entities). 
    /// Collections are referenced in the Toql query language throught a field with a path, say `user_addresses`.
    /// This function can now be used to build a full SQL statement for the collection `addresses`.
    fn build_path(
        &mut self,
        path: &str,
        query: &crate::query::Query,
        wildcard_scope: &WildcardScope,
        cache: &crate::sql_mapper_registry::SqlMapperRegistry,
    ) -> Result<crate::sql_builder_result::SqlBuilderResult, Self::Error>;

    /// Loads all collections for a given struct. This is used by the Toql derive
    /// and issues as many select statements as there merged entities.
    fn load_dependencies(
        &mut self,
        _entities: &mut Vec<T>,
        _entity_keys: &Vec<T::Key>,
        _query: &crate::query::Query,
        _wildcard_scope: &WildcardScope,
        _cache: &crate::sql_mapper_registry::SqlMapperRegistry,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
