
use toql::prelude::{Sql, SqlMapperRegistry, AliasFormat, SqlArg, Result};
use toql::backend::ops::update::Update;
use toql::backend::ops::insert::Insert;
use toql::backend::context::Context;

use std::collections::{HashMap, HashSet};
use toql::backend::ops::conn::Conn;
use toql_core::backend::paths::Paths;
use toql::tree::tree_map::TreeMap;
use toql::tree::tree_predicate::TreePredicate;
use toql::tree::tree_update::TreeUpdate;
use toql::tree::tree_identity::TreeIdentity;
use toql::tree::tree_insert::TreeInsert;
 use toql::sql_mapper::mapped::Mapped;

pub struct Mock {
    pub sqls: Vec<Sql>,
    registry: SqlMapperRegistry,
    affected_rows: u8,
    context: Context

}

impl Mock {

    pub fn clear(&mut self) {
        self.sqls.clear();
    }

    pub fn affected_rows(&mut self, number: u8) {
        self.affected_rows = number;
    }
    
}


impl Default for Mock {
    fn default() -> Self {
        Mock {
        sqls: Vec::new(),
        registry: SqlMapperRegistry::new(),
        affected_rows: 0,
        context : Context {
            roles: HashSet::new(),
            alias_format: AliasFormat::Canonical,
            aux_params:HashMap::new()
            }
        }
    }
    
}

// Implement template functions for updating entities
impl Conn for Mock {
    
   fn execute_sql(&mut self, sql:Sql) -> Result<()> {
        self.sqls.push(sql);
        Ok(())
   }
   fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>> {
        self.sqls.push(sql);
        let ids = (1..=self.affected_rows).map(|n| SqlArg::U64((n + 100).into())).collect::<Vec<_>>();
        Ok(ids)
   }

   fn context(&self) -> &Context {
       &self.context
   }

   fn registry(&self) -> &SqlMapperRegistry {
       &self.registry
   }

   fn registry_mut(&mut self) -> &mut SqlMapperRegistry {
       &mut self.registry
   }
}


impl Insert for Mock {} // Add Insert functionality to Mock
