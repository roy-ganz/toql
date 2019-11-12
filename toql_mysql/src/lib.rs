//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::prelude::GenericConnection;
use toql_core::error::ToqlError;
use toql_core::mutate::Mutate;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;

use toql_core::log_sql;

pub mod load;
pub mod row;
pub mod select;
pub use mysql; // Reexport for derive produced code


fn execute_update_delete_sql <C>(statement: (String, Vec<String>),conn: &mut C ) -> Result<u64, ToqlError> 
where   C: GenericConnection
{
            let (update_stmt, params) = statement;
            log::info!("SQL `{}` with params {:?}", update_stmt, params);
            let mut stmt = conn.prepare(&update_stmt)?;
            let res = stmt.execute(params)?;
            Ok(res.affected_rows())
}

fn execute_insert_sql <C>(statement: (String, Vec<String>),conn: &mut C ) -> Result<u64, ToqlError> 
where   C: GenericConnection
{
             let (insert_stmt, params) = statement;
            log::info!("SQL `{}` with params {:?}", insert_stmt, params);
            let mut stmt = conn.prepare(&insert_stmt)?;
            let res = stmt.execute(params)?;
            Ok(res.last_insert_id())
}

/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_one<'a, T, C>(entity: &'a T, conn: &mut C) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::insert_one_sql(&entity)?;
    execute_insert_sql(sql, conn)
    
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_many<'a, I, T, C>(entities: I, conn: &mut C) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + 'a,
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::insert_many_sql(entities)?;

    Ok(if let Some(sql) = sql {
        execute_insert_sql(sql, conn)?
    } else {
        0
    })
}

/// Delete a struct.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_one<'a, T, C>(entity: &'a T, conn: &mut C) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::delete_one_sql(&entity)?;
     execute_update_delete_sql(sql, conn)
}

/// Delete a collection of structs.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_many<'a, I, T, C>(entities: I, conn: &mut C) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + 'a,
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::delete_many_sql(entities)?;

    Ok(if let Some(sql) = sql {
         execute_update_delete_sql(sql, conn)?
       /*  log_sql!(delete_stmt, params);
        let mut stmt = conn.prepare(delete_stmt)?;
        let res = stmt.execute(params)?;
        res.affected_rows() */
    } else {
        0
    })
}

/// Update a collection of structs.
///
/// Optional fields with value `None` are not updated. See guide for details.
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of updated rows.
pub fn update_many<'a, I, T, C>(entities: I, conn: &mut C) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + Clone + 'a,
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::update_many_sql(entities)?;

    Ok(if let Some(sql) = sql {
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

pub fn update_one<'a, T, C>(entity: &'a T, conn: &mut C) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql = T::update_one_sql(&entity)?;

    Ok(if let Some(sql) = sql {
           execute_update_delete_sql(sql, conn)?
    } else {
        0
    })
}

/// Updates difference of many tuples that contain an outdated and current struct..
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_many<'a, I, T, C>(entities: I, conn: &mut C) -> Result<u64, ToqlError>
where
    I: Iterator<Item = (&'a T, &'a T)> + Clone + 'a,
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    let sql_stmts = T::diff_many_sql(entities)?;
    Ok(if let Some(sql_stmts) = sql_stmts {
        let mut affected = 0u64;
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
pub fn diff_one<'a, T, C>(outdated: &'a T, current: &'a T, conn: &mut C) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
    C: GenericConnection,
{
    diff_many(std::iter::once((outdated, current)), conn)
}



/// Updates difference of two collections.
/// This will insert / update / delete database rows.
/// Nested fields themself will not automatically be updated.
pub fn diff_one_collection<'a, T, C>(outdated: &'a Vec<T>, updated: &'a Vec<T>, conn: &mut C) -> Result<(u64, u64, u64), ToqlError>
where
    T: toql_core::mutate::Mutate<'a, T> + 'a +  toql_core::key::Key,
    C: GenericConnection,
{
      let (insert_sql, diff_sql, delete_sql) = toql_core::diff::collection_delta_sql::<'a, T>(outdated, updated)?;
      let mut affected = (0,0,0);

    if let Some(insert_sql) = insert_sql{
        affected.0 += execute_update_delete_sql(insert_sql, conn)?;
    }
    if let Some(diff_sql) = diff_sql {
        affected.1 += execute_update_delete_sql(diff_sql, conn)?;
    }
    if let Some(delete_sql)= delete_sql{
        affected.2 += execute_update_delete_sql(delete_sql, conn)?;
    }

    Ok(affected)
  
}


/// Selects a single struct for a given key.
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_one<T, C>(key: &<T as toql_core::key::Key>::Key, conn: &mut C) -> Result<T, ToqlError>
where
    T: select::Select<T> + toql_core::key::Key,
    C: GenericConnection,
{
    T::select_one(key, conn)
}

/* /// Selects many structs for a given key. (DOENS)
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_many<T>( key: &<T as toql_core::key::Key<T>>::Key,conn: &mut Conn, first: u64,max: u16) -> Result<Vec<T> , ToqlError>
where T : select::Select<T> + toql_core::key::Key<T>
{
    T::select_many(key, conn, first, max)
} */

/// Load a struct with dependencies for a given Toql query.
///
/// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
pub fn load_one<T, C>(query: &Query, mappers: &SqlMapperCache, conn: &mut C) -> Result<T, ToqlError>
where
    T: load::Load<T>,
    C: GenericConnection,
{
    T::load_one(query, mappers, conn)
}

/// Load a vector of structs with dependencies for a given Toql query.
///
/// Returns a tuple with the structs and an optional tuple of count values.
/// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
/// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
pub fn load_many<T, C>(
    query: &Query,
    mappers: &SqlMapperCache,
    conn: &mut C,
    count: bool,
    first: u64,
    max: u16,
) -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>
where
    T: load::Load<T>,
    C: GenericConnection,
{
    T::load_many(query, mappers, conn, count, first, max)
}
