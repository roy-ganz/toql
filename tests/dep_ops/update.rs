
use toql::prelude::{Sql, SqlMapperRegistry, AliasFormat, SqlArg, Result};
use toql::backend::ops::update::Update;

use std::collections::{HashMap, HashSet};


pub struct TestUpdate {
    pub sqls: Vec<Sql>,
    registry: SqlMapperRegistry,
    roles: HashSet<String>,
    alias_format: AliasFormat,
    aux_params: HashMap<String, SqlArg>
}

impl TestUpdate {

    pub fn clear(&mut self) {
        self.sqls.clear();
    }
    
}


impl Default for TestUpdate {
    fn default() -> Self {
        TestUpdate {
        sqls: Vec::new(),
        registry: SqlMapperRegistry::new(),
        roles: HashSet::new(),
        alias_format: AliasFormat::Canonical,
        aux_params: HashMap::new()
        }

    }
}

// Implement template functions for updating entities
impl<T> Update<T> for TestUpdate {
    fn registry(&self) -> &SqlMapperRegistry {
       &self.registry
   }
    fn registry_mut(&mut self) -> &mut SqlMapperRegistry {
       &mut self.registry
   }
    fn roles(&self) -> &HashSet<String> {
       &self.roles
   }
    fn alias_format(&self) -> AliasFormat {
       self.alias_format.clone()
   }
   fn aux_params(&self) -> &HashMap<String, SqlArg> {
       &self.aux_params
   }
   fn execute_sql(&mut self, sql:Sql) -> Result<()> {
        self.sqls.push(sql);
        Ok(())
   }
}