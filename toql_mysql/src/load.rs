use mysql::prelude::GenericConnection;
use mysql::Conn;
use toql_core::error::ToqlError;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;

/// Trait to load entities from MySQL database.
pub trait Load<T> {
    /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn load_one<C: GenericConnection>(
        query: &Query,
        mappers: &SqlMapperCache,
        conn: &mut C,
    ) -> Result<T, ToqlError>;

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    fn load_many<C: GenericConnection>(
        query: &Query,
        mappers: &SqlMapperCache,
        conn: &mut C,
        count: bool,
        first: u64,
        max: u16,
    ) -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>;
}
