
use toql::prelude::{Sql, SqlMapperRegistry, AliasFormat, SqlArg, Result};
use std::collections::{HashMap, HashSet};
use toql::backend::{Backend, context::Context};
use toql::sql_builder::build_result::BuildResult;

use async_trait::async_trait;

pub struct Mock {
    pub sqls: Vec<Sql>,
    registry: SqlMapperRegistry,
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
        registry: SqlMapperRegistry::new(),
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
impl Backend for Mock {
    
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

    async fn select_sql<T>(&mut self, sql:Sql) -> Result<Vec<T>> {
        self.sqls.push(sql);
        Ok(vec![])
    }
    fn prepare_page(&self, _result: &mut BuildResult, _start:u64, _page_size: u16) {
        
    }

   async  fn select_page_sql<T>(&mut self, sql:Sql) -> Result<(Vec<T>, u32)> {
        self.sqls.push(sql);
        Ok((vec![], 0))
   }
   async fn select_count_sql(&mut self, sql:Sql) -> Result<u32> {
       self.sqls.push(sql);
       Ok(0)
   } 

   
   fn registry(&self) -> &SqlMapperRegistry {
       &self.registry
   }

   fn registry_mut(&mut self) -> &mut SqlMapperRegistry {
       &mut self.registry
   }

   fn roles(&self) -> &HashSet<String> { &self.context.roles }
   fn alias_format(&self) -> AliasFormat { self.context.alias_format.clone() }
   fn aux_params(&self) -> &HashMap<String, SqlArg> { &self.context.aux_params }
}



