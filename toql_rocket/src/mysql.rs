
use crate::toql_query::ToqlQuery;
use toql_core::query::Query;
use toql_core::error::ToqlError;
use toql_core::sql_mapper::SqlMapperCache;
use toql_mysql::load::Load;

/// Facade function to query structs with URL query parameters from a MySQL database.
/// 
/// This needs the MySQL feature enabled.
pub fn load_many<'a, T: Load<T>>(
    toql_query: &ToqlQuery,
    mappers: &SqlMapperCache,
    mut conn: &mut mysql::Conn
) 
-> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>
{
    // Returns sql errors
    T::load_many(
        &toql_query.query.as_ref().unwrap_or(&Query::wildcard()),
        &mappers,
        &mut conn,
        toql_query.count.unwrap_or(true),
        toql_query.first.unwrap_or(0),
        toql_query.max.unwrap_or(10),
    )
}

