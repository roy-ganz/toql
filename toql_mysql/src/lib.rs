//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::prelude::GenericConnection;

use toql_core::mutate::collection_delta_sql;

use toql_core::mutate::{Insert, Update, Diff, Delete, InsertDuplicate, DuplicateStrategy};
use toql_core::load::{Load, Page};
use toql_core::select::Select;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::key::Key;

use toql_core::log_sql;
use core::borrow::Borrow;

use toql_core::sql_builder_result::SqlBuilderResult;

use std::collections::HashSet;

//pub mod diff;
//pub mod insert;
pub mod row;
//pub mod select;
pub use mysql; // Reexport for derive produced code

pub mod error;
use crate::error::Result;
use crate::error::ToqlMySqlError;


fn execute_update_delete_sql<C>(
    statement: (String, Vec<String>),
    conn: &mut C,
) -> Result<u64>
where
    C: GenericConnection,
{
    let (update_stmt, params) = statement;
    log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(params)?;
    Ok(res.affected_rows())
}

fn execute_insert_sql<C>(statement: (String, Vec<String>), conn: &mut C) -> Result<u64>
where
    C: GenericConnection,
{
    let (insert_stmt, params) = statement;
    log_sql!(insert_stmt, params);
    let mut stmt = conn.prepare(&insert_stmt)?;
    let res = stmt.execute(params)?;
    Ok(res.last_insert_id())
}



pub struct MySql<'a, C:GenericConnection>{
    conn: &'a mut C, 
    roles: HashSet<String>
}



impl<C:GenericConnection> MySql<'_,C>{


/// Create connection wrapper from MySql connection or transaction.
///
/// Use the connection wrapper to access all Toql functionality.
pub fn from<'a>(conn: &'a mut C) -> MySql<'a,C> {
    
     MySql{
        conn,
        roles: HashSet::new()
    }
}

/// Create connection wrapper from MySql connection or transaction and roles.
///
/// Use the connection wrapper to access all Toql functionality.
pub fn with_roles<'a>(conn: &'a mut C, roles: HashSet<String>) -> MySql<'a,C> {
    
     MySql{
        conn,
        roles,
    }
}

/// Set roles
///
/// After setting the roles all Toql functions are validated against these roles.
/// Roles on fields can be used to restrict the access (Only super admin can see this field, only group admin can update this field),
pub fn set_roles(&mut self, roles: HashSet<String>) ->&mut Self{
    self.roles= roles;
    self
}

pub fn conn(&mut self) -> &'_ mut C {
    self.conn
}
pub fn roles(&self) -> &HashSet<String> {
    &self.roles
}

/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_one<'a,T>(&mut self, entity: &T) -> Result<u64>
where
   
    Self:  Insert<'a,T, error = ToqlMySqlError>,
    T : 'a
{
    
    let sql = <Self as Insert<'a, T>>::insert_one_sql(entity, DuplicateStrategy::Fail, &self.roles)?;
    execute_insert_sql(sql, self.conn)
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_many<'a, T,Q >(&mut self, entities: &[Q]) -> Result<u64>
where
    Self:  Insert<'a, T, error = ToqlMySqlError>,
    T: 'a,
    Q: Borrow<T>
{
    let sql = <Self as Insert<'a, T>>::insert_many_sql(&entities, DuplicateStrategy::Fail, &self.roles)?;
  
    Ok(if let Some(sql) = sql {
        execute_insert_sql(sql, self.conn)?
    } else {
        0
    })
}
/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_dup_one<'a, T>(&mut self, entity: &T, strategy: DuplicateStrategy) -> Result<u64>
where
    T: 'a,
   Self:  Insert<'a,T, error = ToqlMySqlError> + InsertDuplicate,
{
    let sql =  <Self as Insert<'a, T>>::insert_one_sql(entity, strategy, &self.roles)?;
     
    execute_insert_sql(sql, self.conn)
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_dup_many<'a, T: 'a, I, Q>(&mut self,entities: &[Q], strategy: DuplicateStrategy) -> Result<u64>
where
   Self:  Insert<'a,T, error = ToqlMySqlError> + InsertDuplicate,
   I: 'a,
   Q: Borrow<T>
    
{
    let sql = <Self as Insert<'a, T>>::insert_many_sql(&entities, strategy, &self.roles)?;
     
    Ok(if let Some(sql) = sql {
        
        execute_insert_sql(sql, self.conn)?
    } else {
        0
    })
}

/// Delete a struct.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_one<'a, T>(&mut self, key: <T as Key>::Key) -> Result<u64>
where
   toql_core::dialect::Generic: Delete<'a,T, error = toql_core::error::ToqlError>,
   T: Key + 'a
   
  
{
    let sql =  <toql_core::dialect::Generic as Delete<'a,T>>::delete_one_sql(key, &self.roles)?;
     
    execute_update_delete_sql(sql, self.conn)
}

/// Delete a collection of structs.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_many<'a, T>(&mut self, keys: &[<T as Key>::Key]) -> Result<u64>
where
  T: Key + 'a,
  toql_core::dialect::Generic: Delete<'a,T,error = toql_core::error::ToqlError>
    
{
    
    let sql =  <toql_core::dialect::Generic as Delete<'a,T>>::delete_many_sql(&keys, &self.roles)?;

    Ok(if let Some(sql) = sql {
        
        execute_update_delete_sql(sql, self.conn)?
       } else {
        0
    })
}

/// Update a collection of structs.
///
/// Optional fields with value `None` are not updated. See guide for details.
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of updated rows.
pub fn update_many<'a, T,Q>(&mut self,entities: &[Q]) -> Result<u64>
where
   toql_core::dialect::Generic: Update<'a,T,error = toql_core::error::ToqlError>,
   T: 'a,
   Q: Borrow<T>
{
    let sql = <toql_core::dialect::Generic as Update<'a,T>>::update_many_sql(&entities, &self.roles)?;

    Ok(if let Some(sql) = sql {
         
        execute_update_delete_sql(sql, self.conn)?
    /* log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(params)?;

    res.affected_rows() */
    } else {
        0
    })
    /*   let mut x = 0;

    for entity in entities{
        x += update_one(entity, conn)?
    }
    Ok(x) */
}

/// Update a single struct.
///
/// Optional fields with value `None` are not updated. See guide for details.
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of updated rows.
///

pub fn update_one<'a, T>(&mut self,entity: &T) -> Result<u64>
where
    toql_core::dialect::Generic: Update<'a,T,error = toql_core::error::ToqlError>,
    T:'a
   
{
    let sql = <toql_core::dialect::Generic as Update<'a,T>>::update_one_sql(entity, &self.roles)?;

    Ok(if let Some(sql) = sql {
          
        execute_update_delete_sql(sql, self.conn)?
    } else {
        0
    })
}

/// Updates difference of many tuples that contain an outdated and current struct..
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_many<'a, T, Q: 'a + Borrow<T>>(&mut self,entities: &[(Q, Q)]) -> Result<u64>
where
 Self:  Diff<'a,T,error = ToqlMySqlError>,
 T:'a
   
  
{
    let sql_stmts = <Self as Diff<'a,T>>::diff_many_sql(entities, &self.roles)?;
    Ok(if let Some(sql_stmts) = sql_stmts {
        let mut affected = 0u64;
          
        for sql_stmt in sql_stmts {
            affected += execute_update_delete_sql(sql_stmt, self.conn)?;
            /* let (update_stmt, params) = statements;
            log::info!("SQL `{}` with params {:?}", update_stmt, params);
            let mut stmt = conn.prepare(&update_stmt)?;
            let res = stmt.execute(params)?;
            affected += res.affected_rows(); */
        }
        affected
    } else {
        0
    })
}

/// Updates difference of two struct.
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_one<'a, T>(&mut self, outdated: &'a T, current: &'a T) -> Result<u64>
where
    Self:  Diff<'a,T,error = ToqlMySqlError>,
    T:'a
{
    self.diff_many(&[(outdated, current)])
}

/// Updates difference of two collections.
/// This will insert / update / delete database rows.
/// Nested fields themself will not automatically be updated.
pub fn diff_one_collection<'a, T>(
    &mut self,
    outdated: &'a [T], 
    updated:  &'a [T],
) -> Result<(u64, u64, u64)>
where
 toql_core::dialect::Generic: Delete<'a,T, error= toql_core::error::ToqlError>, 
  Self:   Diff<'a,T,error= ToqlMySqlError> + Insert<'a,T, error= ToqlMySqlError>,
  T: Key + 'a + Borrow<T> 
      
{
    
    let (insert_sql, diff_sql, delete_sql) = collection_delta_sql::<T,Self,Self, toql_core::dialect::Generic, ToqlMySqlError>(outdated, updated, &self.roles)?;
    let mut affected = (0, 0, 0);
      

    if let Some(insert_sql) = insert_sql {
        affected.0 += execute_update_delete_sql(insert_sql, self.conn)?;
    }
    if let Some(diff_sql) = diff_sql {
        affected.1 += execute_update_delete_sql(diff_sql, self.conn)?;
    }
    if let Some(delete_sql) = delete_sql {
        affected.2 += execute_update_delete_sql(delete_sql, self.conn)?;
    }
   

    Ok(affected)
}

/// Selects a single struct for a given key.
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_one<T>(&mut self,key: <T as Key>::Key) -> Result<T>
where
    Self:  Select<T, error = ToqlMySqlError> ,
    T: Key,
   
{
   <Self as Select<T>>::select_one(self, key)
}


/// Selects a single struct for a given key.
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_many<T>(&mut self,keys: &[<T as Key>::Key]) -> Result<Vec<T>>
where
    Self:  Select<T, error = ToqlMySqlError> ,
    T: Key,
   
{
   <Self as Select<T>>::select_many(self, keys)
}

/* /// Selects many structs for a given key. (DOENS)
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_many<T>( key: &<T as Key<T>>::Key,conn: &mut Conn, first: u64,max: u16) -> Result<Vec<T> >
where T : select::Select<T> + Key<T>
{
    T::select_many(key, conn, first, max)
} */

/// Load a struct with dependencies for a given Toql query.
///
/// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
pub fn load_one<T>(&mut self,query: &Query, mappers: &SqlMapperCache) -> Result<T>
where
    Self:  Load<T,error = ToqlMySqlError> , T: toql_core::key::Key
  
{
     <Self as Load<T>>::load_one(self, query, mappers)
    
}

/// Load a vector of structs with dependencies for a given Toql query.
///
/// Returns a tuple with the structs and an optional tuple of count values.
/// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
/// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
pub fn load_many<T>(
    &mut self,
    query: &Query,
    mappers: &SqlMapperCache,
    page: Page,
    
) -> Result<(Vec<T>, Option<(u32, u32)>)>
where
     Self:  Load<T, error = ToqlMySqlError> , T: toql_core::key::Key
{
   <Self as Load<T>>::load_many(self, query, mappers,page)
}
}




/// Helper function to convert result from SQlBuilder into SQL (MySql dialect).
 pub fn sql_from_query_result(result :&SqlBuilderResult, hint: &str, offset: u64, max: u16) -> String {
        let mut s = String::from("SELECT ");

        if !hint.is_empty() {
            s.push_str(hint);
            s.push(' ');
        }

        result.sql_body(&mut s);
        s.push_str(" LIMIT ");
        s.push_str(&offset.to_string());
        s.push(',');
        s.push_str(&max.to_string());

        s
    }