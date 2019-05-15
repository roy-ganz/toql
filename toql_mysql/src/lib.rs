//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements 
//! the trait [Indelup](../toql_core/indelup/trait.Indelup.html).
//! 

use mysql::Conn;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::query::Query;
use toql_core::error::ToqlError;
use toql_core::indelup::Indelup;


pub mod load;
pub mod row;


    /// Insert one struct. 
    /// Skip fields in struct that are auto generated with #[toql(skip_inup)].
    /// Returns the last generated id.
 pub fn insert_one<'a, T>( entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where T:'a + Indelup<'a, T>
 {
     let (insert_stmt, params) = T::insert_one_sql(&entity)?;
     if params.is_empty() {return Ok(0);}
    log::info!("Sql `{}` with params {:?}", insert_stmt, params);
    let mut stmt = conn.prepare(insert_stmt)?;
    let res= stmt.execute(params)?;
    Ok(res.last_insert_id())
     
 }
     
    /// Insert a collection of structs. 
    /// Skip fields in struct that are auto generated with #[toql(skip_inup)].
    /// Returns the last generated id
  pub fn insert_many<'a, I, T > (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where I: Iterator<Item = &'a T> + 'a, T:'a + Indelup<'a, T>
     {
        let (insert_stmt, params) = T::insert_many_sql(entities)?;
        if params.is_empty() {return Ok(0);}
        log::info!("Sql `{}` with params {:?}", insert_stmt, params);
        let mut stmt = conn.prepare(insert_stmt)?;
        let res= stmt.execute(params)?;
        Ok(res.last_insert_id())
    }

    /// Delete a struct. 
    /// The field that is used as key must be attributed with #[toql(delup_key)].
    /// Returns the number of deleted rows.
    pub fn delete_one<'a, T >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where T:'a + Indelup<'a, T>
    {
        let (delete_stmt, params) = T::delete_one_sql(&entity)?;
        log::info!("Sql `{}` with params {:?}", delete_stmt, params);

        let mut stmt = conn.prepare(delete_stmt)?;
        let res = stmt.execute(params)?;
        Ok(res.affected_rows())
        
    }
    /// Delete a collection of structs. 
    /// The field that is used as key must be attributed with #[toql(delup_key)].
    /// Returns the number of deleted rows.
    pub fn delete_many<'a, I, T> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where I: Iterator<Item = &'a T> + 'a ,  T:'a + Indelup<'a, T>
    {
        let (delete_stmt, params)= T::delete_many_sql(entities)?;
        if params.is_empty() {return Ok(0);}
        log::info!("Sql `{}` with params {:?}", delete_stmt, params);
        let mut stmt = conn.prepare(delete_stmt)?;
        let res= stmt.execute(params)?;
        Ok(res.affected_rows())
    }

    /// Update a collection of structs. 
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with #[toql(delup_key)].
    /// Returns the number of updated rows.
    pub fn update_many<'a, I, T> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
        where I: Iterator<Item = &'a T> + 'a,  T:'a + Indelup<'a, T>
         {
            let (update_stmt, params) = T::update_many_sql(&entities)?;
            log::info!("Sql `{}` with params {:?}", update_stmt, params);
            let mut stmt = conn.prepare(&update_stmt)?;
            let res = stmt.execute(params)?;

            Ok(res.affected_rows())
       /*   let mut x = 0;

        for entity in entities{
            x += update_one(entity, conn)?
        }
        Ok(x) */
    }

    /// Update a single struct. 
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with #[toql(delup_key)].
    /// Returns the number of updated rows.
    pub fn update_one<'a, T >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where T:'a + Indelup<'a, T>
    {
        let (update_stmt, params) = T::update_one_sql(&entity)?;
        log::info!("Sql `{}` with params {:?}", update_stmt, params);
        let mut stmt = conn.prepare(&update_stmt)?;
        let res = stmt.execute(params)?;

        Ok(res.affected_rows())
    }
   
/// Load a struct with dependencies for a given Toql query
/// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
 pub fn load_one<T: load::Load<T>> (query: &Query, mappers: &SqlMapperCache, conn: &mut Conn) 
 -> Result<T, ToqlError> {
    T::load_one(query, mappers,conn)
 }

/// Load a collection of struct with dependencies for a given Toql query
/// Returns a tuple with the structs and an optional tuple of count values. 
/// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`.
/// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
 pub fn load_many<T: load::Load<T>>(query: &Query, mappers: &SqlMapperCache, conn: &mut Conn, count: bool, first:u64, max:u16)
-> Result<(Vec<T>, Option<(u32,u32)>), ToqlError>
 {
    T::load_many(query, mappers, conn,  count, first, max)
 }

/* 
 pub fn is_null(row: &mysql::Row, id: usize) -> bool {
    let v : mysql::Value;
    println!("{:?}", row);
    v = row.get(id).unwrap();
    v == mysql::Value::NULL
} */