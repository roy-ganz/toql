//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::prelude::GenericConnection;



use toql_core::mutate::collection_delta_sql;

use toql_core::key::Key;

use toql_core::key::Keyed;
use toql_core::load::{Load, Page};
use toql_core::mutate::{Delete, Diff, DuplicateStrategy, Insert, InsertDuplicate, Update};
use toql_core::query::Query;

use toql_core::sql_mapper_registry::SqlMapperRegistry;

use toql_core::error::ToqlError;
use toql_core::sql_builder::SqlBuilder;

use toql_core::sql_mapper::Mapped;

use core::borrow::Borrow;
use toql_core::log_sql;



use std::collections::HashSet;
use crate::row::FromResultRow;
use crate::row::from_query_result;


//pub mod diff;
//pub mod insert;
pub mod row;
//pub mod select;
pub use mysql; // Reexport for derive produced code

pub mod sql_arg;

pub mod error;
use crate::error::Result;
use crate::error::ToqlMySqlError;
use toql_core::sql::Sql;


use crate::sql_arg::{values_from, values_from_ref};

fn execute_update_delete_sql<C>(statement: Sql, conn: &mut C) -> Result<u64>
where
    C: GenericConnection,
{
    let (update_stmt, params) = statement;
    log_sql!(update_stmt, params);
    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute( values_from(params))?;
    Ok(res.affected_rows())
}

fn execute_insert_sql<C>(statement: Sql, conn: &mut C) -> Result<u64>
where
    C: GenericConnection,
{
    let (insert_stmt, params) = statement;
    log_sql!(insert_stmt, params);

    
    let mut stmt = conn.prepare(&insert_stmt)?;
    let res = stmt.execute(values_from(params))?;
    Ok(res.last_insert_id())
}

pub struct MySql<'a, C: GenericConnection> {
    conn: &'a mut C,
    roles: HashSet<String>,
    registry: &'a SqlMapperRegistry
}

impl<C: GenericConnection> MySql<'_, C> {
    /// Create connection wrapper from MySql connection or transaction.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn from<'a>(conn: &'a mut C, registry: &'a SqlMapperRegistry) -> MySql<'a, C> {
        MySql {
            conn,
            roles: HashSet::new(),
            registry
        }
    }

    /// Create connection wrapper from MySql connection or transaction and roles.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn with_roles<'a>(conn: &'a mut C, registry: &'a SqlMapperRegistry, roles: HashSet<String>) -> MySql<'a, C> {
        MySql { conn, registry,roles }
    }

    /// Set roles
    ///
    /// After setting the roles all Toql functions are validated against these roles.
    /// Roles on fields can be used to restrict the access (Only super admin can see this field, only group admin can update this field),
    pub fn set_roles(&mut self, roles: HashSet<String>) -> &mut Self {
        self.roles = roles;
        self
    }

    pub fn conn(&mut self) -> &'_ mut C {
        self.conn
    }

    pub fn registry(& self) -> &SqlMapperRegistry {
        &self.registry
    }
    pub fn roles(&self) -> &HashSet<String> {
        &self.roles
    }

    /// Insert one struct.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id.
    pub fn insert_one<'a, T>(&mut self, entity: &T) -> Result<u64>
    where
        Self: Insert<'a, T, Error = ToqlMySqlError>,
        T: 'a,
    {
        let sql =
            <Self as Insert<'a, T>>::insert_one_sql(entity, DuplicateStrategy::Fail, &self.roles)?;
        execute_insert_sql(sql, self.conn)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    pub fn insert_many<'a, T, Q>(&mut self, entities: &[Q]) -> Result<u64>
    where
        Self: Insert<'a, T, Error = ToqlMySqlError>,
        T: 'a,
        Q: Borrow<T>,
    {
        let sql = <Self as Insert<'a, T>>::insert_many_sql(
            &entities,
            DuplicateStrategy::Fail,
            &self.roles,
        )?;

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
        T: 'a + InsertDuplicate,
        Self: Insert<'a, T, Error = ToqlMySqlError>,
    {
        let sql = <Self as Insert<'a, T>>::insert_one_sql(entity, strategy, &self.roles)?;

        execute_insert_sql(sql, self.conn)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    pub fn insert_dup_many<'a, T: 'a,  Q>(
        &mut self,
        entities: &[Q],
        strategy: DuplicateStrategy,
    ) -> Result<u64>
    where T: InsertDuplicate,
        Self: Insert<'a, T, Error = ToqlMySqlError> ,
        //I: 'a,
        Q: Borrow<T>,
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
    /// pub fn select_one<K>(&mut self, key: K) -> Result<<K as Key>::Entity>
    
    
    pub fn delete_one<'a, K>(&mut self, key: K) -> Result<u64>
   where
        K: Key + Into<Query<<K as Key>::Entity>>,
        <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
        
    {
        
        let sql_mapper = self.registry.mappers.get( &<K as Key>::Entity::type_name() )
                    .ok_or( ToqlError::MapperMissing(<K as Key>::Entity::type_name()))?;
        
        let query =  Query::from(key);
         let sql = SqlBuilder::new().build_delete_sql(sql_mapper, &query, self.roles())?;

        execute_update_delete_sql(sql, self.conn)
    }

    /// Delete a collection of structs.
    ///
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of deleted rows.
    pub fn delete_many<'a, T>(&mut self, query: &Query<T>) -> Result<u64>
    where
        T: toql_core::sql_mapper::Mapped + 'a,
    {

       let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

         let sql = SqlBuilder::new().build_delete_sql(sql_mapper, query, self.roles())?;

        execute_update_delete_sql(sql, self.conn)
    }

    /// Update a collection of structs.
    ///
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of updated rows.
    pub fn update_many<'a, T, Q>(&mut self, entities: &[Q]) -> Result<u64>
    where
        toql_core::dialect::Generic: Update<'a, T, Error = toql_core::error::ToqlError>,
        T: 'a,
        Q: Borrow<T>,
    {
        let sql = <toql_core::dialect::Generic as Update<'a, T>>::update_many_sql(
            &entities,
            &self.roles,
        )?;

        Ok(if let Some(sql) = sql {
            execute_update_delete_sql(sql, self.conn)?
      
        } else {
            0
        })
    }

    /// Update a single struct.
    ///
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of updated rows.
    ///

    pub fn update_one<'a, T>(&mut self, entity: &T) -> Result<u64>
    where
        toql_core::dialect::Generic: Update<'a, T, Error = toql_core::error::ToqlError>,
        T: 'a,
    {
        let sql =
            <toql_core::dialect::Generic as Update<'a, T>>::update_one_sql(entity, &self.roles)?;

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
    pub fn full_diff_many<'a, T, Q: 'a + Borrow<T>>(&mut self, entities: &[(Q, Q)]) -> Result<u64>
    where
        Self: Diff<'a, T, Error = ToqlMySqlError>,
        T: 'a,
    {
        let sql_stmts = <Self as Diff<'a, T>>::full_diff_many_sql(entities, &self.roles)?;
        Ok(if let Some(sql_stmts) = sql_stmts {
            let mut affected = 0u64;

            for sql_stmt in sql_stmts {
                affected += execute_update_delete_sql(sql_stmt, self.conn)?;
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
    pub fn full_diff_one<'a, T>(&mut self, outdated: &'a T, current: &'a T) -> Result<u64>
    where
        Self: Diff<'a, T, Error = ToqlMySqlError>,
        T: 'a,
    {
        self.full_diff_many(&[(outdated, current)])
    }

    /// Updates difference of many tuples that contain an outdated and current struct..
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be inserted, updated or deleted.
    /// Nested fields themself will not automatically be updated.
    pub fn diff_many<'a, T, Q: 'a + Borrow<T>>(&mut self, entities: &[(Q, Q)]) -> Result<u64>
    where
        Self: Diff<'a, T, Error = ToqlMySqlError>,
        T: 'a,
    {
        let sql_stmts = <Self as Diff<'a, T>>::diff_many_sql(entities, &self.roles)?;
        Ok(if let Some((update_stmt, params)) = sql_stmts {
            log_sql!(update_stmt, params);
            let mut stmt = self.conn.prepare(&update_stmt)?;
            let res = stmt.execute(values_from(params))?;
            res.affected_rows()
        } else {
            0
        })
    }

    /// Updates difference of two struct.
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be ignored.
    pub fn diff_one<'a, T>(&mut self, outdated: &'a T, current: &'a T) -> Result<u64>
    where
        Self: Diff<'a, T, Error = ToqlMySqlError>,
        T: 'a,
    {
        self.diff_many(&[(outdated, current)])
    }

    /// Updates difference of two collections.
    /// This will insert / update / delete database rows.
    /// Nested fields themself will not automatically be updated.
    pub fn diff_one_collection<'a, T>(
        &mut self,
        outdated: &'a [T],
        updated: &'a [T],
    ) -> Result<(u64, u64, u64)>
    where
        toql_core::dialect::Generic: Delete<'a, T, Error = toql_core::error::ToqlError>,
        Self: Diff<'a, T, Error = ToqlMySqlError> + Insert<'a, T, Error = ToqlMySqlError>,
        T: Keyed + Mapped + 'a + Borrow<T>,
    {
        let (insert_sql, diff_sql, delete_sql) =
            collection_delta_sql::<T, Self, Self, toql_core::dialect::Generic, ToqlMySqlError>(
                outdated,
                updated,
                &self.roles,
            )?;
        let mut affected = (0, 0, 0);

        if let Some(insert_sql) = insert_sql {
            affected.0 += execute_insert_sql(insert_sql, self.conn)?;
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
    pub fn select_one<K>(&mut self, key: K) -> Result<<K as Key>::Entity>
    where
        K: Key + Into<Query<<K as Key>::Entity>>,
        <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
    {
     

        let sql_mapper = self.registry.mappers.get( &<<K as Key>::Entity as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<<K as Key>::Entity as Mapped>::type_name()))?;
        let query = Query::from(key);
         let sql = SqlBuilder::new().build_select_sql(sql_mapper,  &query, self.roles(), "", "LIMIT 0,2")?;

         let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
         let mut entities = from_query_result::<<K as Key>::Entity>(entities_stmt)?;

        if entities.len() > 1 {
            return Err(ToqlMySqlError::ToqlError(
                ToqlError::NotUnique,
            ));
        } else if entities.is_empty() {
            return Err(ToqlMySqlError::ToqlError(
                ToqlError::NotFound,
            ));
        }
        Ok(entities.pop().unwrap())
    
    }

    /// Selects a single struct for a given key.
    /// This will select all base fields and joins. Merged fields will be skipped
    pub fn select_many<T>(&mut self, query: &Query<T>) -> Result<Vec<T>>
    where
        T: crate::row::FromResultRow<T> + toql_core::sql_mapper::Mapped, 
    {
        
        let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let sql = SqlBuilder::new().build_select_sql(sql_mapper, query, self.roles(), "", "")?;
       
        log_sql!(sql.0, sql.1);

        let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
        let entities = from_query_result::<T>(entities_stmt)?;
       
        Ok(entities)
    }

    /// Selects all mutable fields of a single struct for a given key.
    /// This will select all base fields and join. Merged fields will be skipped
    pub fn select_mut_one<K>(&mut self, key: K) -> Result<<K as Key>::Entity>
    where
        K: Key + Into<Query<<K as Key>::Entity>>,
        <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
    {
     

        let sql_mapper = self.registry.mappers.get( &<<K as Key>::Entity as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<<K as Key>::Entity as Mapped>::type_name()))?;
        let query = Query::from(key);
         let sql = SqlBuilder::new().build_select_mut_sql(sql_mapper,  &query, self.roles(), "", "LIMIT 0,2")?;

         let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
         let mut entities = from_query_result::<<K as Key>::Entity>(entities_stmt)?;

        if entities.len() > 1 {
            return Err(ToqlMySqlError::ToqlError(
                ToqlError::NotUnique,
            ));
        } else if entities.is_empty() {
            return Err(ToqlMySqlError::ToqlError(
                ToqlError::NotFound,
            ));
        }
        Ok(entities.pop().unwrap())
    
    }

    /// Selects all mutable fields of a single struct for a given key.
    /// This will select all base fields and joins. Merged fields will be skipped
    pub fn select_mut_many<T>(&mut self, query: &Query<T>) -> Result<Vec<T>>
    where
        T: crate::row::FromResultRow<T> + toql_core::sql_mapper::Mapped, 
    {
        
        let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let sql = SqlBuilder::new().build_select_mut_sql(sql_mapper, query, self.roles(), "", "")?;
       
        log_sql!(sql.0, sql.1);

        let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
        let entities = from_query_result::<T>(entities_stmt)?;
       
        Ok(entities)
    }

    

     /// Counts the number of rows that match the query predicate.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn count<T>(&mut self, query: &Query<T>) -> Result<u64>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed + toql_core::sql_mapper::Mapped,
    {
      

        let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let sql = SqlBuilder::new().build_count_sql(sql_mapper, query, self.roles())?;
  
        log_sql!(sql.0, sql.1);
         let result = self.conn.prep_exec(&sql.0, values_from_ref(&sql.1))?;

        let count = result.into_iter().next().unwrap().unwrap().get(0).unwrap();

       Ok(count)
    }


    /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn load_one<T>(&mut self, query: &Query<T>) -> Result<T>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        
        <Self as Load<T>>::load_one(self, query)
    }

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    pub fn load_many<T>(
        &mut self,
        query: &Query<T>,
    ) -> Result<Vec<T>>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        let (r, _) =<Self as Load<T>>::load_many(self, query,None)?;
        Ok(r)
    }

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    pub fn load_page<T>(
        &mut self,
        query: &Query<T>,
        page: Page,
    ) -> Result<(Vec<T>, Option<(u32, u32)>)>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        <Self as Load<T>>::load_many(self, query, Some(page))
    }
}
/* 
/// Helper function to convert result from SQlBuilder into SQL (MySql dialect).
pub fn sql_from_query_result(
    result: &SqlBuilderResult,
    modifier: &str,
    page: Option<(u64,u16)>
) -> String {


    let extra = match page {
        Some((offset, max)) => { 
            let mut e = String::from("LIMIT ");
        e.push_str(&offset.to_string());
        e.push(',');
        e.push_str(&max.to_string()); e},
        None => String::from("")
    }
    ;

    result.query_stmt(modifier, &extra)
   
}
 */
