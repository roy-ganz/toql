//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::Conn;
use toql_core::error::ToqlError;
use toql_core::mutate::Mutate;
use toql_core::query::Query;
use toql_core::sql_mapper::SqlMapperCache;

use toql_core::log_sql;

pub mod load;
pub mod row;
pub mod select;
pub use mysql; // Reexport for derive produced code






/// Insert one struct.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id.
pub fn insert_one<'a, T>(entity: &'a T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
{
    let sql = T::insert_one_sql(&entity)?;
    Ok ( if let Some((insert_stmt, params)) = sql {
         log_sql!(insert_stmt, params);
        let mut stmt = conn.prepare(insert_stmt)?;
        let res = stmt.execute(params)?;
        res.last_insert_id()
    } else {
        0}
    )
}

/// Insert a collection of structs.
///
/// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
/// Returns the last generated id
pub fn insert_many<'a, I, T>(entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + 'a,
    T: 'a + Mutate<'a, T>,
{
    let sql = T::insert_many_sql(entities)?;
    
    Ok ( if let Some((insert_stmt, params)) = sql {
        log_sql!(insert_stmt, params);
        let mut stmt = conn.prepare(insert_stmt)?;
        let res = stmt.execute(params)?;
        res.last_insert_id()
    } else {
        0
    })
}

/// Delete a struct.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_one<'a, T>(entity: &'a T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
{
    let sql = T::delete_one_sql(&entity)?;

      Ok( if let Some((delete_stmt, params)) = sql {
           log_sql!(delete_stmt, params);

            let mut stmt = conn.prepare(delete_stmt)?;
            let res = stmt.execute(params)?;
            res.affected_rows()
      } else {
          0
      })
}
/// Delete a collection of structs.
///
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of deleted rows.
pub fn delete_many<'a, I, T>(entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + 'a,
    T: 'a + Mutate<'a, T>,
{
    let sql = T::delete_many_sql(entities)?;

    Ok( if let Some((delete_stmt, params)) = sql {

       log_sql!(delete_stmt, params);
        let mut stmt = conn.prepare(delete_stmt)?;
        let res = stmt.execute(params)?;
        res.affected_rows()
    } else {
        0
    })
}

/// Update a collection of structs.
///
/// Optional fields with value `None` are not updated. See guide for details.
/// The field that is used as key must be attributed with `#[toql(delup_key)]`.
/// Returns the number of updated rows.
pub fn update_many<'a, I, T>(entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    I: Iterator<Item = &'a T> + Clone + 'a,
    T: 'a + Mutate<'a, T>,
{
    let sql = T::update_many_sql(entities)?;

    Ok( if let Some((update_stmt, params)) = sql {
    log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(params)?;

    res.affected_rows()
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



pub fn update_one<'a, T>(entity: &'a T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    T: 'a + Mutate<'a, T>,
{
    let sql = T::update_one_sql(&entity)?;

   Ok( if let Some((update_stmt, params)) = sql {

        log_sql!(update_stmt, params);
        //log::info!(sql_log!(), update_stmt, params);
        let mut stmt = conn.prepare(&update_stmt)?;
        let res = stmt.execute(params)?;
        res.affected_rows()
    } else {
        0
    }
   )
}

/// Updates difference of many tuples that contain an outdated and current struct..
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_many<'a, I, T>(entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError>
where
    I: Iterator<Item = (&'a T, &'a T)> + Clone + 'a,
    T: 'a + Mutate<'a, T>,
{
    let sql = T::diff_many_sql(entities)?;
    Ok( if let Some(statements) = sql {
        let mut affected = 0u64;
        for statements in statements {
            let (update_stmt,params ) = statements;
            log::info!("SQL `{}` with params {:?}", update_stmt, params);
            let mut stmt = conn.prepare(&update_stmt)?;
            let res = stmt.execute(params)?;
            affected += res.affected_rows();
        }
        affected

    } else {
        0})
    

}

/// Updates difference of two struct.
/// This will updated struct fields and foreign keys from joins.
/// Collections in a struct will be inserted, updated or deleted.
/// Nested fields themself will not automatically be updated.
pub fn diff_one<'a, T>(outdated: &'a T, current: &'a T, conn: &mut Conn) -> Result<u64, ToqlError>
where  T: 'a + Mutate<'a, T>
{
    diff_many(std::iter::once((outdated, current)), conn)

}



/// Selects a single struct for a given key.
/// This will select all base fields and join. Merged fields will be skipped
pub fn select_one<T>(key: &<T as toql_core::key::Key<T>>::Key, conn: &mut Conn) -> Result<T, ToqlError>
where T : select::Select<T> + toql_core::key::Key<T>
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
pub fn load_one<T: load::Load<T>>(
    query: &Query,
    mappers: &SqlMapperCache,
    conn: &mut Conn,
) -> Result<T, ToqlError> {
    T::load_one(query, mappers, conn)
}

/// Load a vector of structs with dependencies for a given Toql query.
///
/// Returns a tuple with the structs and an optional tuple of count values.
/// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
/// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
pub fn load_many<T: load::Load<T>>(
    query: &Query,
    mappers: &SqlMapperCache,
    conn: &mut Conn,
    count: bool,
    first: u64,
    max: u16,
) -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError> {
    T::load_many(query, mappers, conn, count, first, max)
}
