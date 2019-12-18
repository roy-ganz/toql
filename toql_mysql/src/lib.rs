//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::prelude::GenericConnection;
use toql_core::error::ToqlError;
use toql_core::mutate::collection_delta_sql;

use toql_core::mutate::{Insert, Update, Diff, Delete, InsertDuplicate, DuplicateStrategy};
use toql_core::load::Load;
use toql_core::select::Select;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;
use toql_core::key::Key;

use toql_core::log_sql;
use core::borrow::Borrow;

//pub mod diff;
//pub mod insert;
pub mod row;
//pub mod select;
pub use mysql; // Reexport for derive produced code




fn execute_update_delete_sql<C>(
    statement: (String, Vec<String>),
    conn: &mut C,
) -> Result<u64, ToqlError>
where
    C: GenericConnection,
{
    let (update_stmt, params) = statement;
    log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(params)?;
    Ok(res.affected_rows())
}

fn execute_insert_sql<C>(statement: (String, Vec<String>), conn: &mut C) -> Result<u64, ToqlError>
where
    C: GenericConnection,
{
    let (insert_stmt, params) = statement;
    log_sql!(insert_stmt, params);
    let mut stmt = conn.prepare(&insert_stmt)?;
    let res = stmt.execute(params)?;
    Ok(res.last_insert_id())
}



pub struct MySql<C:GenericConnection>(pub C);


impl<C:GenericConnection> MySql<C>{
/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_one<'a,T>(&mut self, entity: T) -> Result<u64, ToqlError>
where
   
    Self:  Insert<'a,T>,
    T : 'a
{
    let conn = &mut self.0;
    let sql = <Self as Insert<'a, T>>::insert_one_sql(entity, DuplicateStrategy::Fail)?;
    execute_insert_sql(sql, conn)
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_many<'a, T >(&mut self, entities: Vec<T>) -> Result<u64, ToqlError>
where
    Self:  Insert<'a, T>,
    T: 'a
{
    let sql = <Self as Insert<'a, T>>::insert_many_sql(entities, DuplicateStrategy::Fail)?;
    let conn = &mut self.0;
    Ok(if let Some(sql) = sql {
        execute_insert_sql(sql, conn)?
    } else {
        0
    })
}
/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_dup_one<'a, T>(&mut self, entity:  T, strategy: DuplicateStrategy) -> Result<u64, ToqlError>
where
    T: 'a,
   Self:  Insert<'a,T> + InsertDuplicate,
    
{
    let sql =  <Self as Insert<'a, T>>::insert_one_sql(entity, strategy)?;
     let conn = &mut self.0;
    execute_insert_sql(sql, conn)
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_dup_many<'a, T: 'a, I>(&mut self,entities: Vec<T>, strategy: DuplicateStrategy) -> Result<u64, ToqlError>
where
   Self:  Insert<'a,T> + InsertDuplicate,
   I: 'a,
   T: Borrow<T>
    
{
    let sql = <Self as Insert<'a, T>>::insert_many_sql(entities, strategy)?;
     
    Ok(if let Some(sql) = sql {
        let conn = &mut self.0;
        execute_insert_sql(sql, conn)?
    } else {
        0
    })
}

/// Delete a struct.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_one<'a, T>(&mut self, key: <T as Key>::Key) -> Result<u64, ToqlError>
where
   toql_core::dialect::Generic: Delete<'a,T>,
   T: Key + 'a
   
  
{
    let sql =  <toql_core::dialect::Generic as Delete<'a,T>>::delete_one_sql(key)?;
     let conn = &mut self.0;
    execute_update_delete_sql(sql, conn)
}

/// Delete a collection of structs.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_many<'a, T>(&mut self, keys: Vec<<T as Key>::Key>) -> Result<u64, ToqlError>
where
  T: Key + 'a,
  toql_core::dialect::Generic: Delete<'a,T>
    
{
    
    let sql =  <toql_core::dialect::Generic as Delete<'a,T>>::delete_many_sql(keys)?;

    Ok(if let Some(sql) = sql {
        let conn = &mut self.0;
        execute_update_delete_sql(sql, conn)?
       } else {
        0
    })
}

/// Update a collection of structs.
///
/// Optional fields with value `None` are not updated. See guide for details.
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of updated rows.
pub fn update_many<'a, T:>(&mut self,entities: Vec<T>) -> Result<u64, ToqlError>
where
   toql_core::dialect::Generic: Update<'a,T>,
   T: 'a
{
    let sql = <toql_core::dialect::Generic as Update<'a,T>>::update_many_sql(entities)?;

    Ok(if let Some(sql) = sql {
         let conn = &mut self.0;
        execute_update_delete_sql(sql, conn)?
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

pub fn update_one<'a, T>(&mut self,entity: T) -> Result<u64, ToqlError>
where
    toql_core::dialect::Generic: Update<'a,T>,
    T:'a
   
{
    let sql = <toql_core::dialect::Generic as Update<'a,T>>::update_one_sql(entity)?;

    Ok(if let Some(sql) = sql {
          let conn = &mut self.0;
        execute_update_delete_sql(sql, conn)?
    } else {
        0
    })
}

/// Updates difference of many tuples that contain an outdated and current struct..
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_many<'a, T>(&mut self,entities: Vec<(T, T)>) -> Result<u64, ToqlError>
where
 Self:  Diff<'a,T>,
 T:'a
   
  
{
    let sql_stmts = <Self as Diff<'a,T>>::diff_many_sql(entities)?;
    Ok(if let Some(sql_stmts) = sql_stmts {
        let mut affected = 0u64;
          let conn = &mut self.0;
        for sql_stmt in sql_stmts {
            affected += execute_update_delete_sql(sql_stmt, conn)?;
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
pub fn diff_one<'a, T>(&mut self, outdated: T, current: T) -> Result<u64, ToqlError>
where
    Self:  Diff<'a,T>,
    T:'a
{
    self.diff_many(vec![(outdated, current)])
}

/// Updates difference of two collections.
/// This will insert / update / delete database rows.
/// Nested fields themself will not automatically be updated.
pub fn diff_one_collection<'a, T>(
    &mut self,
    outdated: Vec<T>,
    updated:  Vec<T>,
) -> Result<(u64, u64, u64), ToqlError>
where
 toql_core::dialect::Generic: Delete<'a,T>, 
  Self:   Diff<'a,T> + Insert<'a,T>,
  T: Key + 'a
 //T: Delete<'a,T> +  Insert<'a,T> + Update<'a, T> + Key + 'a,
      
{
    let (insert_sql, diff_sql, delete_sql) = collection_delta_sql::<T,Self,Self, toql_core::dialect::Generic>(outdated, updated)?;
    let mut affected = (0, 0, 0);
      let conn = &mut self.0;

    if let Some(insert_sql) = insert_sql {
        affected.0 += execute_update_delete_sql(insert_sql, conn)?;
    }
    if let Some(diff_sql) = diff_sql {
        affected.1 += execute_update_delete_sql(diff_sql, conn)?;
    }
    if let Some(delete_sql) = delete_sql {
        affected.2 += execute_update_delete_sql(delete_sql, conn)?;
    }

    Ok(affected)
}

/// Selects a single struct for a given key.
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_one<T>(&mut self,key: <T as Key>::Key) -> Result<T, ToqlError>
where
    Self:  Select<T> ,
    T: Key,
   
{
   <Self as Select<T>>::select_one(self, key)
}

/* /// Selects many structs for a given key. (DOENS)
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_many<T>( key: &<T as Key<T>>::Key,conn: &mut Conn, first: u64,max: u16) -> Result<Vec<T> , ToqlError>
where T : select::Select<T> + Key<T>
{
    T::select_many(key, conn, first, max)
} */

/// Load a struct with dependencies for a given Toql query.
///
/// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
pub fn load_one<T>(&mut self,query: &Query, mappers: &SqlMapperCache) -> Result<T, ToqlError>
where
    Self:  toql_core::load::Load<T> ,
  
{
     <Self as toql_core::load::Load<T>>::load_one(self, query, mappers)
    
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
    page: toql_core::load::Page,
    
) -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>
where
     Self:  toql_core::load::Load<T> ,
{
   <Self as toql_core::load::Load<T>>::load_many(self, query, mappers,page)
}
}