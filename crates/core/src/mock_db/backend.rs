use crate::alias_format::AliasFormat;
use crate::backend::{context::Context, Backend};
use crate::cache::Cache;
use crate::error::ToqlError;
use crate::result::Result;
use crate::sql::Sql;
use crate::sql_arg::SqlArg;
use crate::table_mapper_registry::TableMapperRegistry;
use crate::{page::Page, sql_builder::build_result::BuildResult};
use std::collections::{HashMap, HashSet};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::log_sql;

use super::row::Row;
use async_trait::async_trait;

pub(crate) struct MockDbBackend<'a> {
    pub(crate) sqls: Vec<Sql>,
    pub(crate) context: Context,
    pub(crate) cache: &'a Cache,
    pub(crate) rows: HashMap<String, Vec<Row>>, // Maps select statements to multiple rows
}

// Implement template functions for updating entities
#[async_trait]
impl<'a> Backend<Row, ToqlError> for MockDbBackend<'a> {
    async fn execute_sql(&mut self, sql: Sql) -> Result<()> {
        log_mut_sql!(&sql);
        self.sqls.push(sql);
        Ok(())
    }
    async fn insert_sql(&mut self, sql: Sql) -> Result<Vec<SqlArg>> {
        log_mut_sql!(&sql);
        let number_of_rows: u64 = *(&(sql.0.as_str()).matches(')').count()) as u64;

        self.sqls.push(sql);
        let ids = (0..number_of_rows)
            .map(|n| SqlArg::U64((n + 100).into()))
            .collect::<Vec<_>>();
        Ok(ids)
    }

    async fn select_sql(&mut self, sql: Sql) -> Result<Vec<Row>> {
        log_sql!(&sql);

        let sql_string = sql.to_unsafe_string();
        self.sqls.push(sql);

        // Return empty rows, if no mocked rows are registerd
        if self.rows.is_empty() {
            return Ok(vec![]);
        }

        let rows = self
            .rows
            .get(&sql_string)
            .cloned()
            .ok_or(ToqlError::NoneError(format!(
                "Missing rows for :`{}`",
                &sql_string
            )))?;

        for r in &rows {
            tracing::event!(tracing::Level::INFO, row = %&r, "Mocking row.");
        }

        Ok(rows)
    }
    fn prepare_page(&self, _result: &mut BuildResult, _page: &Page) {}

    async fn select_max_page_size_sql(&mut self, sql: Sql) -> Result<u64> {
        log_sql!(&sql);
        self.sqls.push(sql);
        Ok(0)
    }
    async fn select_count_sql(&mut self, sql: Sql) -> Result<u64> {
        log_sql!(&sql);
        self.sqls.push(sql);
        Ok(0)
    }

    fn registry(&self) -> std::result::Result<RwLockReadGuard<'_, TableMapperRegistry>, ToqlError> {
        self.cache.registry.read().map_err(ToqlError::from)
    }

    fn registry_mut(
        &mut self,
    ) -> std::result::Result<RwLockWriteGuard<'_, TableMapperRegistry>, ToqlError> {
        self.cache.registry.write().map_err(ToqlError::from)
    }

    fn roles(&self) -> &HashSet<String> {
        &self.context.roles
    }
    fn alias_format(&self) -> AliasFormat {
        self.context.alias_format.clone()
    }
    fn aux_params(&self) -> &HashMap<String, SqlArg> {
        &self.context.aux_params
    }
}
