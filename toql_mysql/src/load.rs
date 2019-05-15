

use toql_core::error::ToqlError;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;
use mysql::Conn;


/// Trait to define interface to 
// They load an entity with all dependencies
pub trait Load<T> {
    fn load_one(query: &Query, mappers: &SqlMapperCache, conn: &mut Conn) 
    -> Result<T, ToqlError>;
    fn load_many(query: &Query, mappers: &SqlMapperCache, conn: &mut Conn, count: bool, first:u64, max:u16) 
        -> Result<(Vec<T>, Option<(u32,u32)>),ToqlError>;
 }