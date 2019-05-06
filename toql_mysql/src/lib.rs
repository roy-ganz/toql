
use mysql::Conn;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::query::Query;
use toql_core::error::ToqlError;
use toql_core::indelup::Indelup;


pub mod load;
pub mod row;



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

  pub fn insert_many_test<'a, I, T > (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where I: IntoIterator<Item = &'a T> + 'a, T:'a + Indelup<'a, T>
     {
        let (insert_stmt, params) = T::insert_many_sql(entities.into_iter())?;
        if params.is_empty() {return Ok(0);}
        log::info!("Sql `{}` with params {:?}", insert_stmt, params);
        let mut stmt = conn.prepare(insert_stmt)?;
        let res= stmt.execute(params)?;
        Ok(res.last_insert_id())
    }

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

    pub fn delete_one<'a, T >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where T:'a + Indelup<'a, T>
    {
        let (delete_stmt, params) = T::delete_one_sql(&entity)?;
        log::info!("Sql `{}` with params {:?}", delete_stmt, params);

        let mut stmt = conn.prepare(delete_stmt)?;
        let res = stmt.execute(params)?;
        Ok(res.affected_rows())
        
    }
    pub fn update_one<'a, T >(entity: &T, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
    where T:'a + Indelup<'a, T>
    {
        let (update_stmt, params) = T::update_one_sql(&entity)?;
        log::info!("Sql `{}` with params {:?}", update_stmt, params);
        let mut stmt = conn.prepare(&update_stmt)?;
        let res = stmt.execute(params)?;

        Ok(res.affected_rows())
    }

    pub fn update_many<'a, I, T> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> 
        where I: Iterator<Item = &'a T> + 'a,  T:'a + Indelup<'a, T>
         {
       
         let mut x = 0;

        for entity in entities{
            x += update_one(entity, conn)?
        }
        Ok(x)
    }

   

    

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
    



 pub fn load_one<T: load::Load<T>> (query: &Query, mappers: &SqlMapperCache, conn: &mut Conn) 
 -> Result<T, ToqlError> {
    T::load_one(query, mappers,conn)
 }

 pub fn load_many<T: load::Load<T>>(query: &Query, mappers: &SqlMapperCache, conn: &mut Conn, count: bool, first:u64, max:u16)
-> Result<(Vec<T>, Option<(u32,u32)>), ToqlError>
 {
    T::load_many(query, mappers, conn,  count, first, max)
 }


 pub fn is_null(row: &mysql::Row, id: usize) -> bool {
    let v : mysql::Value;
    println!("{:?}", row);
    v = row.get(id).unwrap();
    v == mysql::Value::NULL
}