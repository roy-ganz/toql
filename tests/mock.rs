
use toql::prelude::{Sql, SqlMapperRegistry, AliasFormat, SqlArg, Result};
use std::collections::{HashMap, HashSet};
use toql::backend::{Backend, context::Context};



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
impl Backend for Mock {
    
   fn execute_sql(&mut self, sql:Sql) -> Result<()> {
        self.sqls.push(sql);
        Ok(())
   }
   fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>> {
        let number_of_rows :u64 = *(&(sql.0.as_str()).matches(')').count()) as u64; 

        self.sqls.push(sql);
        let ids = (0..number_of_rows).map(|n| SqlArg::U64((n + 100).into())).collect::<Vec<_>>();
        Ok(ids)
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



