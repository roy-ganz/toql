//! Toql Api implementations that are database independend 

//pub mod api;
pub mod context;
pub mod context_builder;
//pub mod fields;
pub mod insert;
//pub mod paths;
pub mod update;
pub mod load;
pub mod delete;
pub mod count;
mod map;




use async_trait::async_trait;

use crate::{
    error::ToqlError,
   table_mapper_registry::TableMapperRegistry, alias_format::AliasFormat, sql_arg::SqlArg, sql::Sql, sql_builder::build_result::BuildResult, page::Page
};
use std::{ collections::{HashMap, HashSet}, sync::{RwLockWriteGuard, RwLockReadGuard}};


/// Backend interface that must be implemented by databases to use the default Toql functions.
/// The Backend is implemented for a Row and Error type
/// It contains database specific callbacks for database independend functions
#[async_trait]
pub trait Backend<R,E> 
where  E: From<ToqlError>
{ 
   /// Return the registry with all table mappers
   fn registry(&self) ->Result<RwLockReadGuard<'_, TableMapperRegistry>, ToqlError>;
   /// Return a mutable registry with all table mappers
   fn registry_mut(&mut self) -> Result<RwLockWriteGuard<'_, TableMapperRegistry>, ToqlError>;
   /// Return roles. These will be used for any role restrictions
   fn roles(&self) -> &HashSet<String>;
   /// Return the active alias format. It is used to build all Sql aliases
   fn alias_format(&self) -> AliasFormat;
   /// Return the aux params. These will be used together with the query aux params to resolve aux params in Sql expressions and handlers
   fn aux_params(&self) -> &HashMap<String, SqlArg>;

   /// Execute a select statement on the database and return a vector of rows 
   async fn select_sql(&mut self, sql:Sql) -> Result<Vec<R>, E>;

   /// Modify a builder result, so that page can be loaded
   /// This is different for each database LIMIT on MySql or LIMIT OFFSET on Postgres, etc.
   fn prepare_page(&self, result: &mut BuildResult,  page: &Page); 

   // Execute a select statement and return number of records without page limitation
   async fn select_max_page_size_sql(&mut self, sql:Sql) -> Result<u64, E>; 
   
   // Execute a count select statement and return a the result 
   async fn select_count_sql(&mut self, sql:Sql) -> Result<u64, E>; 

   // Execute a select statement and return nothing
   async fn execute_sql(&mut self, sql:Sql) -> Result<(), E>;
   
   // Execute an insert statement and return new keys
   async fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>, E>; // New ids in descending order
        
 }

