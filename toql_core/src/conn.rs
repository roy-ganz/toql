pub struct GenericConn;

/* #[cfg(feature = "mysqldb")]
pub struct MySqlConn<C:mysql::prelude::GenericConnection>(pub C);

pub struct GenericConn;

use crate::sql_mapper::SqlMapperCache;
use crate::query::Query;
use crate::error::Result;

use crate::log_sql;

fn execute_update_delete_sql<C>(
    statement: (String, Vec<String>),
    conn: &mut C,
) -> Result<u64>
where
    C: mysql::prelude::GenericConnection,
{
    let (update_stmt, params) = statement;
    log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(params)?;
    Ok(res.affected_rows())
}

impl<C:mysql::prelude::GenericConnection> MySqlConn<C>{
    pub fn delete_one<'a, T>( &mut self,  key: <T as crate::key::Key>::Key) -> Result<u64>
    where
    T: 'a + crate::key::Key , Self:  crate::mutate::Delete<'a,T>
  
{
    let sql = <Self as crate::mutate::Delete<'a, T>>::delete_one_sql(key)?;
    let conn = &mut self.0;
    execute_update_delete_sql(sql, conn)
}

pub fn insert_one<'a, T>( &mut self, entity: &'a T) -> Result<u64>
where
    T: 'a,
    Self:  crate::mutate::Insert<'a,T>,
{
    let sql = <Self as crate::mutate::Insert<'a, T>>::insert_one_sql(&entity, crate::mutate::DuplicateStrategy::Fail)?;
    let conn = &mut self.0;
   // execute_insert_sql(sql, conn)
   Ok(3)
}

pub fn load_one<T>(
    &mut self,
    query: &Query,
    mappers: &SqlMapperCache,
    
) -> Result<T>
where
     Self:  crate::load::Load<T>,
   
{
  <Self as crate::load::Load<T>>::load_one(self, query, mappers)
   
}


pub fn load_many<T>(
    &mut self,
    query: &Query,
    mappers: &SqlMapperCache,
    page: crate::load::Page
    
) -> Result<(std::vec::Vec<T>, std::option::Option<(u32, u32)>)>
where
     Self:  crate::load::Load<T>,
   
{
  <Self as crate::load::Load<T>>::load_many(self, query, mappers, page)
   
}



}
/* 

 impl<C:mysql::prelude::GenericConnection> MySqlConn<C> {
    pub fn load_many<T>(
        &mut self,
        query: &Query,
        mappers: &SqlMapperCache,
        page: Page,
        
    ) -> Result<(Vec<T>, Option<(u32, u32)>)>
    where
        T: crate::load::Load<T>,
        
    {
         <Self as crate::load::Load<T>>::load_many( self, query, mappers, page)
        
    }
}


 */ */