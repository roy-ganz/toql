
use toql::prelude::{Sql, TableMapperRegistry, AliasFormat, SqlArg, Result, ToqlError};
use std::collections::{HashMap, HashSet};
use toql::backend::{Backend, context::Context};
use toql::{page::Page, sql_builder::build_result::BuildResult};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
 


use async_trait::async_trait;

pub struct Mock {
    pub sqls: Vec<Sql>,
    registry: RwLock<TableMapperRegistry>,
    context: Context,
}

impl Mock {

    pub fn clear(&mut self) {
        self.sqls.clear();
    }

   
    
}


impl Default for Mock {
    fn default() -> Self {
        Mock {
        sqls: Vec::new(),
        registry: RwLock::new(TableMapperRegistry::new()),
        context : Context {
            roles: HashSet::new(),
            alias_format: AliasFormat::Canonical,
            aux_params:HashMap::new()
            }
        }
    }
    
}

// Implement template functions for updating entities
#[async_trait]
impl Backend<(), ToqlError> for Mock {
    
   async fn execute_sql(&mut self, sql:Sql) -> Result<()> {
        self.sqls.push(sql);
        Ok(())
   }
   async fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>> {
        let number_of_rows :u64 = *(&(sql.0.as_str()).matches(')').count()) as u64; 

        self.sqls.push(sql);
        let ids = (0..number_of_rows).map(|n| SqlArg::U64((n + 100).into())).collect::<Vec<_>>();
        Ok(ids)
   }

    async fn select_sql(&mut self, sql:Sql) -> Result<Vec<()>> {
        self.sqls.push(sql);
        Ok(vec![])
    }
    fn prepare_page(&self, _result: &mut BuildResult, _page: &Page) {
        
    }

   async  fn select_max_page_size_sql(&mut self, sql:Sql) -> Result<u64> {
        self.sqls.push(sql);
        Ok( 0)
   }
   async fn select_count_sql(&mut self, sql:Sql) -> Result<u64> {
       self.sqls.push(sql);
       Ok(0)
   } 

   
   fn registry(&self) ->std::result::Result<RwLockReadGuard<'_, TableMapperRegistry>, ToqlError> {
       self.registry.read().map_err(ToqlError::from)
   }

   fn registry_mut(&mut self) ->  std::result::Result<RwLockWriteGuard<'_, TableMapperRegistry>, ToqlError>  {
      self.registry.write().map_err(ToqlError::from)
   }

   fn roles(&self) -> &HashSet<String> { &self.context.roles }
   fn alias_format(&self) -> AliasFormat { self.context.alias_format.clone() }
   fn aux_params(&self) -> &HashMap<String, SqlArg> { &self.context.aux_params }
}



