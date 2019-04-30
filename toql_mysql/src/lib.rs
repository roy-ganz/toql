
use mysql::Conn;
use mysql::error::Error;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::query::Query;
use toql_core::error::ToqlError;


pub mod load;
pub mod row;
pub mod alter;


 pub fn insert_one<'a, T:'a + alter::Alter<'a, T>>( entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> {
     T::insert_one(&entity, conn)
 }
    pub fn delete_one<'a, T:'a + alter::Alter<'a, T> >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> {
        T::delete_one(&entity, conn)
    }
    pub fn update_one<'a, T:'a + alter::Alter<'a, T> >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> {
         T::update_one(&entity, conn)
    }



 pub fn load_one<T: load::Load<T>> (query: &mut Query, mappers: &SqlMapperCache, conn: &mut Conn, distinct: bool) 
 -> Result<T, ToqlError> {
    T::load_one(query, mappers,conn, distinct)
 }

 pub fn load_many<T: load::Load<T>>(query: &mut Query, mappers: &SqlMapperCache, conn: &mut Conn, distinct: bool, count: bool, first:u64, max:u16)
-> Result<(Vec<T>, Option<(u32,u32)>), ToqlError>
 {
    T::load_many(query, mappers, conn, distinct, count, first, max)
 }


 pub fn is_null(row: &mysql::Row, id: usize) -> bool {
    let v : mysql::Value;
    println!("{:?}", row);
    v = row.get(id).unwrap();
    v == mysql::Value::NULL
}