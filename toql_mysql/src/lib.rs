//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::prelude::GenericConnection;

use toql_core::mutate::collection_delta_sql;

use toql_core::key::Keyed;
use toql_core::load::{Load, Page};
use toql_core::mutate::{Delete, Diff, DuplicateStrategy, Insert, InsertDuplicate, Update};
use toql_core::query::Query;
use toql_core::select::Select;
use toql_core::sql_mapper_registry::SqlMapperRegistry;
 use toql_core::sql_mapper::SqlMapper;

use toql_core::sql_mapper::Mapped;

use core::borrow::Borrow;
use toql_core::log_sql;

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

fn execute_update_delete_sql<C>(statement: (String, Vec<String>), conn: &mut C) -> Result<u64>
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

pub struct MySql<'a, C: GenericConnection> {
    conn: &'a mut C,
    roles: HashSet<String>,
}

impl<C: GenericConnection> MySql<'_, C> {
    /// Create connection wrapper from MySql connection or transaction.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn from<'a>(conn: &'a mut C) -> MySql<'a, C> {
        MySql {
            conn,
            roles: HashSet::new(),
        }
    }

    /// Create connection wrapper from MySql connection or transaction and roles.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn with_roles<'a>(conn: &'a mut C, roles: HashSet<String>) -> MySql<'a, C> {
        MySql { conn, roles }
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
    pub fn delete_one<'a, T>(&mut self, key: <T as Keyed>::Key) -> Result<u64>
    where
        toql_core::dialect::Generic: Delete<'a, T, Error = toql_core::error::ToqlError>,
        T: Keyed + toql_core::sql_mapper::Mapped + 'a,
        
    {
        let sql = <toql_core::dialect::Generic as Delete<'a, T>>::delete_one_sql(key, &self.roles)?;

        execute_update_delete_sql(sql, self.conn)
    }

    /// Delete a collection of structs.
    ///
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of deleted rows.
    pub fn delete_many<'a, T>(&mut self, predicate: (String, Vec<String>)) -> Result<u64>
    where
        T: Keyed + 'a,
        toql_core::dialect::Generic: Delete<'a, T, Error = toql_core::error::ToqlError>,
    {

        let sql =
            <toql_core::dialect::Generic as Delete<'a, T>>::delete_many_sql(predicate, &self.roles)?;

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
            let res = stmt.execute(params)?;
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
   /*  pub fn select_one<T>(&mut self, key: <T as Key>::Key) -> Result<T>
    where
        Self: Select<T, Error = ToqlMySqlError>,
        T: Key + crate::row::FromResultRow<T> ,
        <T as Key>::Key : toql_core::sql_predicate::SqlPredicate
    { */
    pub fn select_one<K>(&mut self, key: K) -> Result<<K as toql_core::key::Key>::Entity>
    where
        Self: Select<<K as toql_core::key::Key>::Entity, Error = ToqlMySqlError>,
        K: toql_core::key::Key ,
        <K as toql_core::key::Key>::Entity: Keyed + crate::row::FromResultRow<<K as toql_core::key::Key>::Entity>,
    {
        
        use toql_core::select::Select;
        use crate::error::ToqlMySqlError;
        use crate::row::from_query_result;
        use toql_core::error::ToqlError;

        let conn = self.conn();
        let (predicate, params) = toql_core::key::sql_predicate(&[key], &Self::table_alias());
         let select_stmt = format!(
            "{} WHERE {} LIMIT 0,2",
            <Self as Select<<K as toql_core::key::Key>::Entity>>::select_sql(None),
            predicate
        );
        log_sql!(select_stmt, params);

        let entities_stmt = conn.prep_exec(select_stmt, &params)?;
        let mut entities = from_query_result::<<K as toql_core::key::Key>::Entity>(entities_stmt)?;
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
    pub fn select_many<T>(&mut self, predicate: (String, Vec<String>)) -> Result<Vec<T>>
    where
        Self: Select<T, Error = ToqlMySqlError>,
        T: crate::row::FromResultRow<T> + toql_core::key::Keyed, 
        
    {
        use toql_core::sql_predicate::SqlPredicate;
        use toql_core::select::Select;
        use crate::error::ToqlMySqlError;
        use crate::row::from_query_result;
        use toql_core::error::ToqlError;




        let conn = self.conn();

        let (stmt, params) = predicate;
         

        
       // let (predicate, params) = predicate.sql_predicate(&Self::table_alias());
         let select_stmt = format!(
            "{} WHERE {}",
            <Self as Select<T>>::select_sql(None),
            stmt
        );
        println!("{}", select_stmt);
        log_sql!(select_stmt, params);

        let entities_stmt = conn.prep_exec(select_stmt, &params)?;
        let entities = from_query_result::<T>(entities_stmt)?;
       
        Ok(entities)
    
    }

    

   /*  /// Selects a single struct for a given key.
    /// This will select all base fields and join. Merged fields will be skipped
    pub fn select_many<T>(&mut self, keys: &[<T as Key>::Key]) -> Result<Vec<T>>
    where
        Self: Select<T, Error = ToqlMySqlError>,
        T: Key,
    {
        <Self as Select<T>>::select_many(self, keys)
    } */

 /// Selects a single struct for a given key.
    /// This will select all base fields and join. Merged fields will be skipped
   /*  pub fn select_many<T>(&mut self, predicate: T) -> Result<Vec<T>>
    where
        Self: Select<T, Error = ToqlMySqlError>,
        T: Into<toql_core::sql_predicate::SqlPredicate>,
    {
        <Self as Select<T>>::select_many(self, predicate)
    } */

     /// Selects many structs for a given key. (DOENS)
    /// This will select all base fields and join. Merged fields will be skipped
  /*   pub fn select_many<T>( key: &<T as Key<T>>::Key,conn: &mut Conn, first: u64,max: u16) -> Result<Vec<T> >
    where T : select::Select<T> + Key<T>
    {
        T::select_many(key, conn, first, max)
    }  */


     /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn count<T>(&mut self, query: &Query, mapper: &SqlMapper) -> Result<u64>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        
         let mut result = toql_core::sql_builder::SqlBuilder::new()
                    .build(mapper, &query, self.roles()).map_err(|e|toql_core::error::ToqlError::SqlBuilderError(e))?;
         let (stmt, params) = (result.count_stmt(), result.count_params());           
         log_sql!(stmt, params);       
         let result = self.conn.prep_exec(&stmt, &params)?;

        let count = result.into_iter().next().unwrap().unwrap().get(0).unwrap();

       Ok(count)
    }


    /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn load_one<T>(&mut self, query: &Query, registry: &SqlMapperRegistry) -> Result<T>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        <Self as Load<T>>::load_one(self, query, registry)
    }

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    pub fn load_many<T>(
        &mut self,
        query: &Query,
        registry: &SqlMapperRegistry,
        page: Page,
    ) -> Result<(Vec<T>, Option<(u32, u32)>)>
    where
        Self: Load<T, Error = ToqlMySqlError>,
        T: toql_core::key::Keyed,
    {
        <Self as Load<T>>::load_many(self, query, registry, page)
    }
}

/// Helper function to convert result from SQlBuilder into SQL (MySql dialect).
pub fn sql_from_query_result(
    result: &SqlBuilderResult,
    modifier: &str,
    offset: u64,
    max: u16,
) -> String {

    let mut extra = String::from("LIMIT ");
    extra.push_str(&offset.to_string());
    extra.push(',');
    extra.push_str(&max.to_string());

    result.query_stmt(modifier, &extra)
    /* let mut s = String::from("SELECT ");

    if !hint.is_empty() {
        s.push_str(hint);
        s.push(' ');
    }
    s.push_str(&result.select_clause);
    result.sql_body(&mut s);
    s.push_str(" LIMIT ");
    s.push_str(&offset.to_string());
    s.push(',');
    s.push_str(&max.to_string());

    s */
}



pub fn insert_order_clause<K>(keys: &[K], alias:& str) -> String
where K: toql_core::key::Key
{
	
	
    if keys.is_empty() {
    	return String::new();
	}
    let mut clause = String::new();
	for col in K::columns() {
	    	
    	clause.push_str("FIELD(");
    	clause.push_str(alias);
    	clause.push('.');
    	clause.push_str( &col);
        clause.push_str(", ");
    	
     	for k in keys {
        	for a  in k.params() {
            	clause.push_str(&a);
            	clause.push_str(", ");
            }
            clause.pop();
            clause.pop();
	    }
        clause.push_str("), ")
    }
	
	clause.pop();
	clause.pop();

    clause
}

