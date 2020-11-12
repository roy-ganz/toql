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
use toql_core::sql_builder::{sql_builder_error::SqlBuilderError, SqlBuilder};

use core::borrow::Borrow;
use toql_core::alias::AliasFormat;
use toql_core::log_sql;

use crate::row::FromResultRow;
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
};
use toql_core::paths::Paths;
use toql_core::fields::Fields;

//pub mod diff;
//pub mod insert;
pub mod row;
pub mod insert;
pub mod update;

#[macro_use]
pub mod access;

//pub mod select;
pub use mysql; // Reexport for derive produced code

pub mod sql_arg;

pub mod error;
use crate::error::Result;
use crate::error::ToqlMySqlError;
use toql_core::sql::Sql;
use toql_core::sql_arg::SqlArg;
use toql_core::tree::tree_predicate::TreePredicate;
use toql_core::tree::{
    tree_index::TreeIndex, tree_insert::TreeInsert, tree_keys::TreeKeys, tree_merge::TreeMerge, tree_identity::TreeIdentity, tree_update::TreeUpdate,
};
use toql_core::{
    alias_translator::AliasTranslator,
    from_row::FromRow,
    parameter::ParameterMap,
    sql_expr::{PredicateColumn, resolver::Resolver},
    sql_mapper::{mapped::Mapped, SqlMapper},
};

use crate::sql_arg::{values_from, values_from_ref};

fn load_top<T, B, C>(
    mysql: &mut MySql<C>,
    query: &B,
    page: Option<Page>,
) -> Result<(Vec<T>, HashSet<String>)>
where
    T: Keyed
        + Mapped
        + FromRow<Row>
        + TreePredicate
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

    let extra = match page {
        Some(Page::Counted(start, number_of_records)) => {
            Cow::Owned(format!("LIMIT {},{}", start, number_of_records))
        }
        Some(Page::Uncounted(start, number_of_records)) => {
            Cow::Owned(format!("LIMIT {},{}", start, number_of_records))
        }
        None => Cow::Borrowed(""),
    };

    let sql = result
        .to_sql_with_modifier_and_extra(&aux_params, &mut alias_translator, "", extra.borrow())
        .map_err(ToqlError::from)?;
    
    log_sql!(&sql);
    let Sql(sql_stmt, args) = sql;

    let args = crate::sql_arg::values_from_ref(&args);
    let query_results = mysql.conn.prep_exec(sql_stmt, args)?;

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
) -> Result<HashSet<String>>
where
    T: Keyed
        + Mapped
        + FromRow<Row>
        + TreePredicate
        + TreeIndex<Row, ToqlMySqlError>
        + TreeMerge<Row, ToqlMySqlError>,

    B: Borrow<Query<T>>,
    <T as toql_core::key::Keyed>::Key: FromRow<Row>,
    C: GenericConnection,
    ToqlMySqlError: std::convert::From<<T as toql_core::from_row::FromRow<mysql::Row>>::Error>,
{
    use toql_core::sql_expr::SqlExpr;
    use toql_core::sql_expr::PredicateColumn;

    let ty = <T as Mapped>::type_name();
    let mut pending_paths = HashSet::new();

    let mapper = mysql.registry().mappers.get(&ty).ok_or(ToqlError::MapperMissing(ty.clone()))?;
    let merge_base_alias = mapper.canonical_table_alias.clone();

    for root_path in unmerged_paths {
        // Get merge JOIN with ON from mapper
        let mut builder = SqlBuilder::new(&ty, mysql.registry()); // Add alias format or translator to constructor
        let mut result = builder.build_select(root_path.as_str(), query.borrow())?;
        pending_paths = result.unmerged_paths().clone();
       
        let other_alias=  result.table_alias().clone();

        // Build merge join
        // Get merge join and custom on predicate from mapper
        let on_sql_expr = builder.merge_expr(&root_path)?;

        let (merge_join, merge_on) = {
            let merge_resolver = Resolver::new()
            .with_self_alias(&merge_base_alias)
            .with_other_alias(&result.table_alias());
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
        let (_field, ancestor_path) = FieldPath::split_basename(root_path.as_str());
        let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
        let mut d = ancestor_path.descendents();

        let columns =  TreePredicate::columns(entities.get(0).unwrap(), &mut d )
                .map_err(ToqlError::from)?;

        let mut args = Vec::new();
        for e in entities.iter() {
            TreePredicate::args(e, &mut d, &mut args)
                .map_err(ToqlError::from)?;
        }
        let predicate_columns= columns.into_iter().map(|c| PredicateColumn::SelfAliased(c)).collect::<Vec<_>>();
        predicate_expr.push_predicate(predicate_columns , args);

        let predicate_expr = {
            let merge_resolver = Resolver::new()
            .with_self_alias(&merge_base_alias)
            .with_other_alias(other_alias.as_str());
            merge_resolver
                .resolve(&predicate_expr)
                .map_err(ToqlError::from)?
        };
        result.push_join(SqlExpr::literal(" AND "));
        result.push_join(predicate_expr);
        result.push_join(SqlExpr::literal(")"));

        // Build SQL query statement

        let mut alias_translator = AliasTranslator::new(mysql.alias_format());
        let aux_params = [mysql.aux_params()];
        let aux_params = ParameterMap::new(&aux_params);
        let Sql(sql, args) = result
            .to_sql(&aux_params, &mut alias_translator)
            .map_err(ToqlError::from)?;
        dbg!(&sql);
        dbg!(&args);

        // Load from database

        let args = crate::sql_arg::values_from_ref(&args);
        let query_results = mysql.conn.prep_exec(sql, args)?;

        // Build index
       // let row_offset = result.column_counter();
      

        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();

        let (field, ancestor_path) = FieldPath::split_basename(root_path.as_str());
        let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
        let mut d = ancestor_path.descendents();

        // TODO Batch process rows
        // TODO Introduce traits that do not need copy to vec
        let mut rows = Vec::with_capacity(100);

        for q in query_results {
            rows.push(q?); // Stream into Vec
        }

        let row_offset = 0; // key must be forst columns in reow
        <T as TreeIndex<Row, ToqlMySqlError>>::index(&mut d, field, &rows, row_offset, &mut index)?;
        println!("{:?}", result.selection_stream());

        // Merge into entities
        for e in entities.iter_mut() {
            <T as TreeMerge<_, ToqlMySqlError>>::merge(
                e,
                &mut d,
                field,
                &rows,
                row_offset,
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
    pub fn insert_many<T, Q>(&mut self, paths: Paths, mut entities: &mut [Q]) -> Result<u64>
    where
        T: TreeInsert + Mapped + TreeIdentity,
        Q: BorrowMut<T>,
    {
      
       
        // Build up execution tree
        // Path `a_b_merge1_c_d_merge2_e` becomes
        // [0] = [a, c, e]
        // [1] = [a_b, c_d]
        // [m] = [merge1, merge2]
        // Then execution order is [1], [0], [m]

        // TODO should be possible to impl with &str
        let mut joins: Vec<HashSet<String>> = Vec::new();
        let mut merges: Vec<String> = Vec::new();

         crate::insert::build_insert_tree::<T>(&self.registry.mappers, &paths.0, &mut joins, &mut merges)?;

        // Insert root
        let sql = 
                {
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    let home_path = FieldPath::default();
                    
                    crate::insert::build_insert_sql::<T, _>(&self.registry().mappers, self.alias_format(), &aux_params, entities, &home_path, "", "")
                }?;

        log_sql!(&sql);
        dbg!(sql.to_unsafe_string());
        let Sql(insert_stmt, insert_values) = sql;

        let params = values_from(insert_values);
        {
            let mut stmt = self.conn().prepare(&insert_stmt)?;
            let res = stmt.execute(params)?;

            if res.affected_rows() == 0 {
                return Ok(0);
            }

            if <T as toql_core::tree::tree_identity::TreeIdentity>::auto_id() {
                let mut id: u64 =  res.last_insert_id(); // first id
                let home_path = FieldPath::default();
                let mut descendents= home_path.descendents();
                for  e in entities.iter_mut() {
                    {
                    let e_mut = e.borrow_mut();
                    <T as toql_core::tree::tree_identity::TreeIdentity>::set_id( e_mut, &mut descendents,  id)?;
                    }
                    id += 1;
                }
            }
        }

     

        // Insert joins and merges
        for l in (0..joins.len()).rev() {
            for p in joins.get(l).unwrap() {
              
                 let mut path = FieldPath::from(&p);
                
                let sql = 
                {
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::insert::build_insert_sql::<T, _>(&self.registry().mappers, self.alias_format(), &aux_params, entities, &mut path, "", "")
                }?;

                log_sql!(&sql);
                dbg!(sql.to_unsafe_string());
                let Sql(insert_stmt, insert_values) = sql;
                                
                // Execute
                let params = values_from(insert_values);
                let mut stmt = self.conn().prepare(&insert_stmt)?;
                let res = stmt.execute(params)?;

                // set keys
                let path = FieldPath::from(&p);
               let mut descendents = path.descendents();
               crate::insert::set_tree_identity( res.last_insert_id(), &mut entities, &mut descendents)?;
            }
        }
          for p in merges {
              
                let path = FieldPath::from(&p);
              
                let sql = {
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::insert::build_insert_sql::<T, _>(&self.registry().mappers, self.alias_format(), &aux_params, entities, &path, "", "")
                    }?;

                log_sql!(&sql);
                dbg!(sql.to_unsafe_string());
                let Sql(insert_stmt, insert_values) = sql;
                
                // Execute
                let params = values_from(insert_values);
                let mut stmt = self.conn().prepare(&insert_stmt)?;
                stmt.execute(params)?;
                
                // Merges must not contain auto value as identity, skip set_tree_identity
        }

        

        Ok(0)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    /* pub fn insert_many<T, Q>(&mut self, entities: &[Q]) -> Result<u64>
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
    } */

    pub fn insert_one<T>(&mut self, paths: Paths, entity: &mut T) -> Result<u64>
    where
        T: TreeInsert + Mapped + TreeIdentity,
    {
        self.insert_many::<T, _>(paths, &mut [entity])
    }
/* 
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
    } */

     /// Insert one struct.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id.
    pub fn update_many<T, Q>(&mut self, fields: Fields, entities: &mut [Q]) -> Result<()>
    where
        T: TreeUpdate + Mapped + TreeIdentity + TreePredicate,
        Q: BorrowMut<T>,
    {
      use toql_core::sql_expr::{SqlExpr, PredicateColumn};
      
       
        // Build up execution tree
        // Path `a_b_merge1_c_d_merge2_e` becomes
        // [0] = [a, c, e]
        // [1] = [a_b, c_d]
        // [m] = [merge1, merge2]
        // Then execution order is [1], [0], [m]

        // TODO should be possible to impl with &str
        let mut joins: Vec<HashSet<String>> = Vec::new();
        let mut merges: Vec<String> = Vec::new();
        let mut path_fields: HashMap<String, HashSet<String>> = HashMap::new();

        let paths = Vec::new();
        for f in fields.0 {
            let (base, path )  = FieldPath::split_basename(f);
            let p = path.unwrap_or_default();
            if path_fields.get(p.as_str()).is_none() {
                path_fields.insert(p.as_str().to_string(), HashSet::new());
            }
            path_fields.get_mut(p.as_str()).unwrap().insert(base.to_string());
        }

         crate::insert::build_insert_tree::<T>(&self.registry.mappers, &paths, &mut joins, &mut merges)?;
        
        let sqls = 
                {
                  
                    let home_path = FieldPath::default();
                    let default_home_fields=  HashSet::new();
                    let home_fields=  path_fields.get(home_path.as_str()).unwrap_or(&default_home_fields);
                    let canonical_table_alias = <T as Mapped>::table_alias();
                    
                    crate::update::build_update_sql::<T, _>(
                    self.alias_format(), 
                    entities,
                     &home_path, 
                    home_fields,
                    self.roles(),
                     "", "")
                }?;

            // Update base  entities
            for sql in sqls {
                dbg!(sql.to_unsafe_string());
                execute_update_delete_sql(sql, self.conn)?;
            }
           
       
  
       for i in 0..joins.len() {
           for paths in joins.get(i) {
               for path in paths{
                let sqls = 
                {
                  
                    let default_path_fields=  HashSet::new();
                    let path_fields=  path_fields.get(path).unwrap_or(&default_path_fields);
                    let field_path =  FieldPath::from(path);
                    crate::update::build_update_sql::<T, _>(
                    self.alias_format(), 
                    entities,
                    &field_path, 
                    path_fields,
                    self.roles(),
                     "", "")
                }?;

            // Update joins
            for sql in sqls {
                dbg!(sql.to_unsafe_string());
                execute_update_delete_sql(sql, self.conn)?;
            }
           }
           }
       }

        // Delete existing merges and insert new merges
       
        let table_alias= <T as Mapped> ::table_alias();
        for merge in merges {

            let (base, path) = FieldPath::split_basename(&merge);

            // Build delete sql
            let merge_path = path.unwrap_or(FieldPath::default());
            let entity = entities.get(0).unwrap().borrow();
            let columns = <T as TreePredicate>::columns(entity,&mut merge_path.descendents())?;
            let mut args = Vec::new();
            for e in entities.iter(){
                <T as TreePredicate>::args(e.borrow(), &mut merge_path.descendents(), &mut args)?;
            }
            let columns = columns.into_iter().map(|c| PredicateColumn::SelfAliased(c)).collect::<Vec<_>>();
        
            // Construct sql
            let mut key_predicate :SqlExpr = SqlExpr::new();
            key_predicate.push_predicate(columns, args);

            let mut sql_builder = SqlBuilder::new(&table_alias, self.registry());
            let delete_expr = sql_builder.build_merge_delete(&merge_path, key_predicate)?;
            
            let mut alias_translator = AliasTranslator::new(self.alias_format());
            let resolver = Resolver::new();
            let sql = resolver.to_sql(&delete_expr, &mut alias_translator).map_err(ToqlError::from)?;

            dbg!(sql.to_unsafe_string());
            execute_update_delete_sql(sql, self.conn)?;

            // Update keys (TODO)
         /*    for e in  entities.iter_mut(){
                let mut descendents = FieldPath::from(&merge).descendents();
                let e: &mut T = e.borrow_mut();
                <T as TreeIdentity>::set_id(e, &mut descendents, 0)?;
            } */

            



        }

        Ok(())
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
/* 
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
    } */

    /// Update a single struct.
    ///
    /// Optional fields with value `None` are not updated. See guide for details.
    /// The field that is used as key must be attributed with `#[toql(key)]`.
    /// Returns the number of updated rows.
    ///

    pub fn update_one< T>(&mut self, fields: Fields, entity: &mut T) -> Result<()>
    where
        T: TreeUpdate +  Mapped + TreeIdentity + TreePredicate,
       
    {
        
       self.update_many::<T,_>(fields, &mut [entity])
    }

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
