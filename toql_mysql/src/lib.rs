
use mysql::Conn;
use mysql::error::Error;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::query::Query;
 use toql_core::load::LoadError;


pub mod load;
pub mod row;



 pub fn load_one<T: load::Load<T>> (query: &mut Query, mappers: &SqlMapperCache, conn: &mut Conn, distinct: bool) 
 -> Result<T, LoadError> {
    T::load_one(query, mappers,conn, distinct)
 }

 pub fn load_many<T: load::Load<T>>(query: &mut Query, mappers: &SqlMapperCache, conn: &mut Conn, distinct: bool, count: bool, first:u64, max:u16)
-> Result<(Vec<T>, Option<(u32,u32)>), LoadError>
 {
    T::load_many(query, mappers, conn, distinct, count, first, max)
 }


 pub fn is_null(row: &mysql::Row, id: usize) -> bool {
    let v : mysql::Value;
    println!("{:?}", row);
    v = row.get(id).unwrap();
    v == mysql::Value::NULL
}