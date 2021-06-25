
use crate::result::Result;
use crate::sql::Sql;
use crate::{sql_mapper_registry::SqlMapperRegistry, sql_arg::SqlArg, backend::context::Context};

pub trait Conn {
    fn execute_sql(&mut self, sql:Sql) -> Result<()>;
    fn insert_sql(&mut self, sql:Sql) -> Result< Vec<SqlArg>>; // Return new keys

    fn registry(&self) -> &SqlMapperRegistry;
    fn registry_mut(&mut self) -> &mut SqlMapperRegistry;
    fn context(&self) -> &Context;

}