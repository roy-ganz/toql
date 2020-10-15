//!
//! The Toql MySQL integration facade functions to load a struct from a MySQL database and insert, delete and update it.
//! The actual functionality is created by the Toql Derive that implements
//! the trait [Mutate](../toql_core/mutate/trait.Mutate.html).
//!

use mysql::{prelude::GenericConnection, Row};

//use toql_core::mutate::collection_delta_sql;

use toql_core::key::Key;

use toql_core::key::Keyed;
use toql_core::load::Page;
use toql_core::mutate::{DiffSql, DuplicateStrategy, InsertDuplicate, InsertSql, UpdateSql};
use toql_core::query::{field_path::FieldPath, Query};

use toql_core::sql_mapper_registry::SqlMapperRegistry;

use toql_core::error::ToqlError;
use toql_core::sql_builder::SqlBuilder;

use core::borrow::Borrow;
use toql_core::alias::AliasFormat;
use toql_core::log_sql;

use crate::row::FromResultRow;
use std::collections::{HashMap, HashSet};

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
use toql_core::sql_arg::SqlArg;
use toql_core::tree::tree_predicate::TreePredicate;
use toql_core::tree::{tree_index::TreeIndex, tree_keys::TreeKeys, tree_merge::TreeMerge};
use toql_core::{
    alias_translator::AliasTranslator, from_row::FromRow, parameter::ParameterMap,
    sql_expr::resolver::Resolver, sql_mapper::mapped::Mapped,
};

use crate::sql_arg::{values_from, values_from_ref};




fn load_top<T, B, C>(
    mysql: &mut MySql<C>,
    query: &B,
    page: Option<Page>,
) ->  Result<(Vec<T>, HashSet<String>)>
where
    T: Keyed
        + Mapped
        + FromRow<Row>
        + TreePredicate
        + TreeKeys
        + TreeIndex<Row, ToqlMySqlError>
        + TreeMerge<Row, ToqlMySqlError>,
    B: Borrow<Query<T>>,
    <T as toql_core::key::Keyed>::Key: FromRow<Row>,
    C: GenericConnection,
    ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
{
    use std::borrow::Cow;

    let alias_format = mysql.alias_format();

    let ty = <T as Mapped>::type_name();

   
    let mut builder = SqlBuilder::new(&ty, mysql.registry());
    let result = builder.build_select("", query.borrow())?;

    let unmerged = result.unmerged_paths().clone();
    let mut alias_translator = AliasTranslator::new(alias_format);
    let aux_params = [mysql.aux_params()];
    let aux_params = ParameterMap::new(&aux_params);

    let  extra = match page {
        Some(Page::Counted(start, number_of_records)) => Cow::Owned(format!("LIMIT {},{}", start, number_of_records)),
        Some(Page::Uncounted( start, number_of_records)) => Cow::Owned(format!("LIMIT {},{}", start, number_of_records)),
        None => Cow::Borrowed("")
    };

    let Sql(sql, args) = result
        .to_sql_with_modifier_and_extra(&aux_params, &mut alias_translator,"", extra.borrow() )
        .map_err(ToqlError::from)?;

    dbg!(&sql);

    let args = crate::sql_arg::values_from_ref(&args);
    let query_results = mysql.conn.prep_exec(sql, args)?;
    
    let mut entities: Vec<T> = Vec::new();
    for r in query_results {
        let r = r?;
        let mut iter = result.selection_stream().iter();
        let mut i = 0usize;
        entities.push(
            <T as toql_core::from_row::FromRow<mysql::Row>>::from_row_with_index(
                &r, &mut i, &mut iter,
            )?,
        );
       
    }

        Ok((entities, unmerged))
    
}


fn load_and_merge<T, B, C>(
    mysql: &mut MySql<C>,
    query: &B,
    entities: &mut Vec<T>,
    unmerged_paths: &HashSet<String>,
) ->  Result<HashSet<String>>
where
    T: Keyed
        + Mapped
        + FromRow<Row>
        + TreePredicate
        + TreeKeys
        + TreeIndex<Row, ToqlMySqlError>
        + TreeMerge<Row, ToqlMySqlError>,
      
    B: Borrow<Query<T>>,
    <T as toql_core::key::Keyed>::Key: FromRow<Row>,
    C: GenericConnection,
    ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
{
    use toql_core::sql_expr::SqlExpr;

    let ty = <T as Mapped>::type_name();
    let mut pending_paths = HashSet::new();

    for root_path in unmerged_paths {
            // Get merge JOIN with ON from mapper
            let mut builder = SqlBuilder::new(&ty, mysql.registry()); // Add alias format or translator to constructor
            let mut result = builder.build_select(root_path.as_str(), query.borrow())?;
            pending_paths = result.unmerged_paths().clone();

            let resolver = Resolver::new()
            .with_self_alias(&result.table_alias());
          

            // Get merge columns (primary key, merge key)
            // and append to regular select columns
            let mut col_expr = SqlExpr::new();
            let (field, path) = FieldPath::split_basename(&root_path);
            let path = path.unwrap_or(FieldPath::from(""));
            let mut d = path.descendents();
            <T as TreeKeys>::keys(&mut d, field, &mut col_expr).map_err(ToqlError::from)?;
            let col_expr = resolver.resolve(&col_expr).map_err(ToqlError::from)?;
            println!("{}", &col_expr);
            result.push_select(col_expr);

            // Build merge join
            // Get merge join and custom on predicate from mapper
            let on_sql_expr = builder.merge_expr(&root_path)?;

            let (merge_join, merge_on) = {
                let merge_resolver = Resolver::new().with_other_alias(&result.table_alias());
                (
                    merge_resolver
                        .resolve(&on_sql_expr.0)
                        .map_err(ToqlError::from)?,
                    merge_resolver
                        .resolve(&on_sql_expr.1)
                        .map_err(ToqlError::from)?,
                )
            };

            println!("{} ON {}", merge_join, merge_on);
            result.push_join(merge_join);
            result.push_join(SqlExpr::literal("ON ("));
            result.push_join(merge_on);

            // Get ON predicate from entity keys
            let mut predicate_expr = SqlExpr::new();
            let (field, ancestor_path) = FieldPath::split_basename(root_path.as_str());
            let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
            let mut d = ancestor_path.descendents();

            for e in entities.iter() {
                TreePredicate::predicate(e, &mut d, field, &mut predicate_expr)
                    .map_err(ToqlError::from)?;
            }

            let predicate_expr = {
                let merge_resolver = Resolver::new().with_other_alias(&result.table_alias());
                merge_resolver
                    .resolve(&predicate_expr)
                    .map_err(ToqlError::from)?
            };
            result.push_join(SqlExpr::literal("AND "));
            result.push_join(predicate_expr);
            result.push_join(SqlExpr::literal(")"));

            // Build SQL query statement
            result.update_selections_from_placeholders();

            let mut alias_translator = AliasTranslator::new(mysql.alias_format());
            let aux_params = [mysql.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            let Sql(sql, args) = result
                .to_sql(&aux_params, &mut alias_translator)
                .map_err(ToqlError::from)?;
            dbg!(&sql);
            dbg!(&args);

            // Load form database

            let args = crate::sql_arg::values_from_ref(&args);
            let query_results = mysql.conn.prep_exec(sql, args)?;

            // Build index
            let row_offset = result
                .selection_stream()
                .iter()
                .filter(|x| x == &&true)
                .count();
            let mut index: HashMap<u64, Vec<usize>> = HashMap::new();

            let (field, ancestor_path) = FieldPath::split_basename(root_path.as_str());
            let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
            let mut d = ancestor_path.descendents();

            // TODO Batch process rows
            // TODO Introduce traits that do not need copy to vec
            let mut rows = Vec::with_capacity(100);
            println!("affected rows {}", query_results.affected_rows());

            for q in query_results {
                rows.push(q?); // Stream into Vec
            }

            <T as TreeIndex<Row, ToqlMySqlError>>::index(
                &mut d, field, &rows, row_offset, &mut index,
            )?;

            // Merge into entities
            for e in  entities.iter_mut() {
                <T as TreeMerge<_, ToqlMySqlError>>::merge(
                    e,
                    &mut d,
                    field,
                    &rows,
                    &index,
                    result.selection_stream(),
                )?;
            }
        }
        Ok(pending_paths)

}

fn load<T, B, C>(
    mysql: &mut MySql<C>,
    query: B,
    page: Option<Page>,
) -> Result<(Vec<T>, Option<(u32, u32)>)>
where
    T: Keyed
        + Mapped
        + FromRow<Row>
        + TreePredicate
        + TreeKeys
        + TreeIndex<Row, ToqlMySqlError>
        + TreeMerge<Row, ToqlMySqlError>,
    B: Borrow<Query<T>>,
    <T as toql_core::key::Keyed>::Key: FromRow<Row>,
    C: GenericConnection,
    ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
{
  
    let (mut entities, mut unmerged_paths) = load_top(mysql, &query, page)?;
    
    loop {
        let mut pending_paths = load_and_merge(mysql, &query, &mut entities, &unmerged_paths)?;

        // Quit, if all paths have been merged
        if pending_paths.is_empty() {
            break;
        }

        // Select and merge next paths
        unmerged_paths.extend(pending_paths.drain());
    }

    Ok((entities, None))
}

fn execute_update_delete_sql<C>(statement: Sql, conn: &mut C) -> Result<u64>
where
    C: GenericConnection,
{
    log_sql!(&statement);
    let Sql(update_stmt, params) = statement;

    let mut stmt = conn.prepare(&update_stmt)?;
    let res = stmt.execute(values_from(params))?;
    Ok(res.affected_rows())
}

fn execute_insert_sql<C>(statement: Sql, conn: &mut C) -> Result<u64>
where
    C: GenericConnection,
{
    log_sql!(&statement);
    let Sql(insert_stmt, params) = statement;

    let mut stmt = conn.prepare(&insert_stmt)?;
    let res = stmt.execute(values_from(params))?;
    Ok(res.last_insert_id())
}

pub struct MySql<'a, C: GenericConnection> {
    conn: &'a mut C,
    roles: HashSet<String>,
    registry: &'a SqlMapperRegistry,
    aux_params: HashMap<String, SqlArg>,
    alias_format: AliasFormat,
}

impl<'a, C: 'a + GenericConnection> MySql<'a, C> {
    /// Create connection wrapper from MySql connection or transaction.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn from(conn: &'a mut C, registry: &'a SqlMapperRegistry) -> MySql<'a, C> {
        Self::with_roles_and_aux_params(conn, registry, HashSet::new(), HashMap::new())
    }

    /// Create connection wrapper from MySql connection or transaction and roles.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn with_roles(
        conn: &'a mut C,
        registry: &'a SqlMapperRegistry,
        roles: HashSet<String>,
    ) -> MySql<'a, C> {
        Self::with_roles_and_aux_params(conn, registry, roles, HashMap::new())
    }
    /// Create connection wrapper from MySql connection or transaction and roles.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn with_aux_params(
        conn: &'a mut C,
        registry: &'a SqlMapperRegistry,
        aux_params: HashMap<String, SqlArg>,
    ) -> MySql<'a, C> {
        Self::with_roles_and_aux_params(conn, registry, HashSet::new(), aux_params)
    }
    /// Create connection wrapper from MySql connection or transaction and roles.
    ///
    /// Use the connection wrapper to access all Toql functionality.
    pub fn with_roles_and_aux_params(
        conn: &'a mut C,
        registry: &'a SqlMapperRegistry,
        roles: HashSet<String>,
        aux_params: HashMap<String, SqlArg>,
    ) -> MySql<'a, C> {
        MySql {
            conn,
            registry,
            roles,
            aux_params,
            alias_format: AliasFormat::Canonical,
        }
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

    pub fn registry(&self) -> &SqlMapperRegistry {
        &self.registry
    }
    pub fn roles(&self) -> &HashSet<String> {
        &self.roles
    }

    pub fn alias_format(&self) -> AliasFormat {
        self.alias_format.to_owned()
    }

    /* pub fn set_aux_params(&mut self, aux_params: HashMap<String, SqlArg>) -> &mut Self {
           self.aux_params = aux_params;
           self
       }
    */
    pub fn aux_params(&self) -> &HashMap<String, SqlArg> {
        &self.aux_params
    }

    /// Insert one struct.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id.
    pub fn insert_one<T>(&mut self, entity: &T) -> Result<u64>
    where
        T: InsertSql,
    {
        let sql = <T as InsertSql>::insert_one_sql(entity, &self.roles, "", "")?;
        execute_insert_sql(sql, self.conn)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    pub fn insert_many<T, Q>(&mut self, entities: &[Q]) -> Result<u64>
    where
        T: InsertSql,
        Q: Borrow<T>,
    {
        let sql = <T as InsertSql>::insert_many_sql(&entities, &self.roles, "", "")?;

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
    pub fn insert_dup_one<T>(&mut self, entity: &T, strategy: DuplicateStrategy) -> Result<u64>
    where
        T: InsertSql + InsertDuplicate,
    {
        let (modifier, extra) = match strategy {
            DuplicateStrategy::Skip => ("INGNORE", ""),
            DuplicateStrategy::Update => ("", "ON DUPLICATE UPDATE"),
            DuplicateStrategy::Fail => ("", ""),
        };

        let sql = <T as InsertSql>::insert_one_sql(entity, &self.roles, modifier, extra)?;

        execute_insert_sql(sql, self.conn)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    pub fn insert_dup_many<T, Q>(
        &mut self,
        entities: &[Q],
        strategy: DuplicateStrategy,
    ) -> Result<u64>
    where
        T: InsertSql + InsertDuplicate,
        Q: Borrow<T>,
    {
        let (modifier, extra) = match strategy {
            DuplicateStrategy::Skip => ("INGNORE", ""),
            DuplicateStrategy::Update => ("", "ON DUPLICATE UPDATE"),
            DuplicateStrategy::Fail => ("", ""),
        };
        let sql = <T as InsertSql>::insert_many_sql(&entities, &self.roles, modifier, extra)?;

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

    pub fn delete_one<K>(&mut self, key: K) -> Result<u64>
    where
        K: Key + Into<Query<<K as Key>::Entity>>,
        <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
    {
        /*  let sql_mapper = self.registry.mappers.get( &<K as Key>::Entity::type_name() )
        .ok_or( ToqlError::MapperMissing(<K as Key>::Entity::type_name()))?; */

        let query = Query::from(key);

        self.delete_many(query)

        //execute_update_delete_sql(sql, self.conn)
    }

    /// Delete a collection of structs.
    ///
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of deleted rows.
    pub fn delete_many<T, B>(&mut self, query: B) -> Result<u64>
    where
        T: Mapped,
        B: Borrow<Query<T>>,
    {
        /*  let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
        .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?; */

        let result = SqlBuilder::new(&<T as Mapped>::type_name(), self.registry)
            .with_aux_params(self.aux_params().clone()) // todo ref
            .with_roles(self.roles().clone()) // todo ref
            .build_delete(query.borrow())?;

        // No arguments, nothing to delete
        if result.is_empty() {
            Ok(0)
        } else {
            let pa = [&self.aux_params];
            let p = ParameterMap::new(&pa);
            let mut alias_translator = AliasTranslator::new(self.alias_format());
            let sql = result
                .to_sql(&p, &mut alias_translator)
                .map_err(ToqlError::from)?;
            execute_update_delete_sql(sql, self.conn)
        }
    }

    /// Update a collection of structs.
    ///
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with `#[toql(delup_key)]`.
    /// Returns the number of updated rows.
    pub fn update_many<T, Q>(&mut self, entities: &[Q]) -> Result<u64>
    where
        T: UpdateSql,
        Q: Borrow<T>,
    {
        let sql = <T as UpdateSql>::update_many_sql(&entities, &self.roles)?;

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

    pub fn update_one<T>(&mut self, entity: &T) -> Result<u64>
    where
        T: UpdateSql,
    {
        let sql = <T as UpdateSql>::update_one_sql(entity, &self.roles)?;

        Ok(if let Some(sql) = sql {
            execute_update_delete_sql(sql, self.conn)?
        } else {
            0
        })
    }

   /*  /// Updates difference of many tuples that contain an outdated and current struct..
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be inserted, updated or deleted.
    /// Nested fields themself will not automatically be updated.
    pub fn full_diff_many<T, Q: Borrow<T>>(&mut self, entities: &[(Q, Q)]) -> Result<u64>
    where
        T: DiffSql + Mapped,
    {
        let sql_mapper = self
            .registry
            .mappers
            .get(&<T as Mapped>::type_name())
            .ok_or(ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let sql_stmts = <T as DiffSql>::full_diff_many_sql(entities, &self.roles, sql_mapper)?;
        Ok(if let Some(sql_stmts) = sql_stmts {
            let mut affected = 0u64;

            for sql_stmt in sql_stmts {
                affected += execute_update_delete_sql(sql_stmt, self.conn)?;
            }
            affected
        } else {
            0
        })
    } */

    /* /// Updates difference of two struct.
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be inserted, updated or deleted.
    /// Nested fields themself will not automatically be updated.
    pub fn full_diff_one<T>(&mut self, outdated: &T, current: &T) -> Result<u64>
    where
        T: DiffSql + Mapped,
    {
        self.full_diff_many::<T, _>(&[(outdated, current)])
    }

    /// Updates difference of many tuples that contain an outdated and current struct..
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be inserted, updated or deleted.
    /// Nested fields themself will not automatically be updated.
    pub fn diff_many<T, Q: Borrow<T>>(&mut self, entities: &[(Q, Q)]) -> Result<u64>
    where
        T: DiffSql,
    {
        let sql_stmts = <T as DiffSql>::diff_many_sql(entities, &self.roles)?;
        Ok(if let Some(sql) = sql_stmts {
            log_sql!(&sql);
            let Sql(update_stmt, params) = sql;
            let mut stmt = self.conn.prepare(&update_stmt)?;
            let res = stmt.execute(values_from(params))?;
            res.affected_rows()
        } else {
            0
        })
    } */

    /* /// Updates difference of two struct.
    /// This will updated struct fields and foreign keys from joins.
    /// Collections in a struct will be ignored.
    pub fn diff_one<T>(&mut self, outdated: &T, current: &T) -> Result<u64>
    where
        T: DiffSql,
    {
        self.diff_many::<T, _>(&[(outdated, current)])
    }

    /// Updates difference of two collections.
    /// This will insert / update / delete database rows.
    /// Nested fields themself will not automatically be updated.
    pub fn diff_one_collection<T>(
        &mut self,
        outdated: &[T],
        updated: &[T],
    ) -> Result<(u64, u64, u64)>
    where
        T: Keyed + Mapped + Borrow<T> + DiffSql + InsertSql + UpdateSql,
        <T as Keyed>::Key: toql_core::to_query::ToQuery<T>,
    {
        let sql_mapper = self
            .registry
            .mappers
            .get(&<T as Mapped>::type_name())
            .ok_or(ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let (insert_sql, diff_sql, delete_sql) = collection_delta_sql::<T>(
            outdated,
            updated,
            self.roles.clone(),
            self.registry,
            self.alias_format.clone(),
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
    } */

    /*  /// Selects a single struct for a given key.
       /// This will select all base fields and join. Merged fields will be skipped
       pub fn select_one<K>(&mut self, key: K) -> Result<<K as Key>::Entity>
       where
           K: Key + Into<Query<<K as Key>::Entity>>,
           <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
       {


           let sql_mapper = self.registry.mappers.get( &<<K as Key>::Entity as Mapped>::type_name() )
                       .ok_or( ToqlError::MapperMissing(<<K as Key>::Entity as Mapped>::type_name()))?;
           let query = Query::from(key);
            let sql = SqlBuilder::new(self.aux_params()).build_select_all_sql(sql_mapper,  &query, self.roles(), "", "LIMIT 0,2")?;

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
       pub fn select_many<T, B>(&mut self, query: B) -> Result<Vec<T>>
       where
           T: crate::row::FromResultRow<T> + toql_core::sql_mapper::mapped::Mapped,
           B: Borrow<Query<T>>
       {

           let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                       .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

           let sql = SqlBuilder::new( &<T as Mapped>::type_name(), self.registry )
           .with_roles(self.roles().clone())
           .with_aux_params(self.aux_params().clone())
           .build_select_all_sql(sql_mapper, query.borrow(), self.roles(), "", "")?;

           log_sql!(sql.0, sql.1);

           let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
           let entities = from_query_result::<T>(entities_stmt)?;

           Ok(entities)
       }
    */
    /*  /// Selects all mutable fields of a single struct for a given key.
    /// This will select all base fields and join. Merged fields will be skipped
    pub fn select_mut_one<K>(&mut self, key: K) -> Result<<K as Key>::Entity>
    where
        K: Key + Into<Query<<K as Key>::Entity>>,
        <K as Key>::Entity: FromResultRow<<K as Key>::Entity> + Mapped,
    {


        let sql_mapper = self.registry.mappers.get( &<<K as Key>::Entity as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<<K as Key>::Entity as Mapped>::type_name()))?;
        let query = Query::from(key);
         let sql = SqlBuilder::new(self.aux_params()).build_select_mut_sql(sql_mapper,  &query, self.roles(), "", "LIMIT 0,2")?;

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

    } */

    /* /// Selects all mutable fields of a single struct for a given key.
    /// This will select all base fields and joins. Merged fields will be skipped
    pub fn select_mut_many<T, B>(&mut self, query: B) -> Result<Vec<T>>
    where
        T: crate::row::FromResultRow<T> + toql_core::sql_mapper::mapped::Mapped,
        B: Borrow<Query<T>>
    {

        let sql_mapper = self.registry.mappers.get( &<T as Mapped>::type_name() )
                    .ok_or( ToqlError::MapperMissing(<T as Mapped>::type_name()))?;

        let (sql, deser, un_fields) = SqlBuilder::new(&<T as Mapped>::type_name(), self.registry)
        .with_roles(self.roles().clone())
        .build_select_sql(&<T as Mapped>::type_name(), query.borrow(),  "", "", self.alias_format)?;

        log_sql!(&sql);

        let entities_stmt = self.conn.prep_exec(sql.0, values_from_ref(&sql.1))?;
        let entities = from_query_result::<T>(entities_stmt)?;

        Ok(entities)
    } */

    /// Counts the number of rows that match the query predicate.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn count<T, B>(&mut self, query: B) -> Result<u64>
    where
        T: toql_core::key::Keyed + toql_core::sql_mapper::mapped::Mapped,
        B: Borrow<Query<T>>,
    {
        /* let sql_mapper = self
        .registry
        .mappers
        .get(&<T as Mapped>::type_name())
        .ok_or(ToqlError::MapperMissing(<T as Mapped>::type_name()))?; */

        let mut alias_translator = AliasTranslator::new(self.alias_format());

        let result = SqlBuilder::new(&<T as Mapped>::type_name(), self.registry)
            .with_roles(self.roles().clone())
            .with_aux_params(self.aux_params().clone())
            .build_count("", query.borrow())?;
        let p = [self.aux_params()];
        let aux_params = ParameterMap::new(&p);

        let sql = result
            .to_sql(&aux_params, &mut alias_translator)
            .map_err(ToqlError::from)?;

        log_sql!(sql);
        let result = self.conn.prep_exec(&sql.0, values_from_ref(&sql.1))?;

        let count = result.into_iter().next().unwrap().unwrap().get(0).unwrap();

        Ok(count)
    }

    /// Load a struct with dependencies for a given Toql query.
    ///
    /// Returns a struct or a [ToqlMySqlError](../toql_core/error/enum.ToqlMySqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    pub fn load_one<T, B>(&mut self, query: B) -> Result<T>
    where
        T: Keyed
            + Mapped
            + FromRow<Row>
            + TreePredicate
            + TreeKeys
            + TreeIndex<Row, ToqlMySqlError>
            + TreeMerge<Row, ToqlMySqlError>,
        B: Borrow<Query<T>>,
        <T as Keyed>::Key: FromRow<Row>,
        ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
    {
        // <Self as Load<T>>::load_one(self, query.borrow())
        let (mut e, _) = load(self, query.borrow(), Some(Page::Uncounted(0, 2)))?;
        match e.len() {
            0 => Err(ToqlError::NotFound.into()),
            1 => Ok(e.pop().unwrap()),
            _ => Err(ToqlError::NotUnique.into()),
        }
    }

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    pub fn load_many<T, B>(&mut self, query: B) -> Result<Vec<T>>
    where
        T: toql_core::key::Keyed
            + toql_core::sql_mapper::mapped::Mapped
            + FromRow<Row>
            + TreePredicate
            + TreeKeys
            + TreeIndex<Row, ToqlMySqlError>
            + TreeMerge<Row, ToqlMySqlError>,
        B: Borrow<Query<T>>,
        <T as toql_core::key::Keyed>::Key: FromRow<Row>,
        ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
    {
        let (entities, _) = load(self, query, None)?;
        Ok(entities)
    }

    /// Load a vector of structs with dependencies for a given Toql query.
    ///
    /// Returns a tuple with the structs and an optional tuple of count values.
    /// If `count` argument is `false`, no count queries are run and the resulting `Option<(u32,u32)>` will be `None`
    /// otherwise the count queries are run and it will be `Some((total count, filtered count))`.
    pub fn load_page<T, B>(&mut self, query: B, page: Page) -> Result<(Vec<T>, Option<(u32, u32)>)>
    where
        T: Keyed
            + Mapped
            + FromRow<Row>
            + TreePredicate
            + TreeKeys
            + TreeIndex<Row, ToqlMySqlError>
            + TreeMerge<Row, ToqlMySqlError>,
        B: Borrow<Query<T>>,
        <T as Keyed>::Key: FromRow<Row>,
        ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
    {
        let entities_page = load(self, query.borrow(), Some(page))?;
        Ok(entities_page)
    }
}
