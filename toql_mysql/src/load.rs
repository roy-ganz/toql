

use toql_core::load::LoadError;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;
use mysql::PooledConn;
use mysql::error::Error;

// High level convenience functions
// They load an entity with all dependencies
pub trait Load<T> {
    fn load_one(query: &mut Query, mappers: &SqlMapperCache, conn: &mut PooledConn, distinct: bool) 
    -> Result<T, LoadError>;
    fn load_many(query: &mut Query, mappers: &SqlMapperCache, conn: &mut PooledConn, distinct: bool, count: bool, first:u64, max:u16) 
        -> Result<(Vec<T>, Option<(u32,u32)>),Error>;
 }