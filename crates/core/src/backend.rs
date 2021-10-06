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
#[async_trait]
pub trait Backend<R,E> 
where  E: From<ToqlError>
{ 
   fn registry(&self) ->Result<RwLockReadGuard<'_, TableMapperRegistry>, ToqlError>;
   fn registry_mut(&mut self) -> Result<RwLockWriteGuard<'_, TableMapperRegistry>, ToqlError>;
   fn roles(&self) -> &HashSet<String>;
   fn alias_format(&self) -> AliasFormat;
   fn aux_params(&self) -> &HashMap<String, SqlArg>;

   async fn select_sql(&mut self, sql:Sql) -> Result<Vec<R>, E>;
   fn prepare_page(&self, result: &mut BuildResult,  page: &Page); // Modify result, so that page with unlimited page size can be loaded
   async fn select_max_page_size_sql(&mut self, sql:Sql) -> Result<u64, E>; // Load number of records without page limitation
   async fn select_count_sql(&mut self, sql:Sql) -> Result<u64, E>; // Load single value

   async fn execute_sql(&mut self, sql:Sql) -> Result<(), E>;
   async fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>, E>; // New ids in descending order
        
 }

