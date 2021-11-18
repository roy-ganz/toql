//! The Toql Mock Db provides a dummy database that can be used for testing or documentation examples.

use crate::backend::context::Context;
use crate::cache::Cache;
use std::collections::HashMap;

pub mod backend;
#[macro_use]
pub mod row;

pub mod toql_api;

use backend::MockDbBackend;

///
/// The implementation collects all SQL statements that can be asserted.
/// For loading it return default values.
///
pub struct MockDb<'a> {
    backend: MockDbBackend<'a>,
}

impl<'a> MockDb<'a> {
    pub fn clear_rows(&mut self) {
        self.backend.rows.clear();
    }
    pub fn take_unsafe_sqls(&mut self) -> Vec<String> {
        self.clear_rows();
        self.backend
            .sqls
            .drain(..)
            .map(|s| s.to_unsafe_string())
            .collect::<Vec<_>>()
    }
    pub fn take_unsafe_sql(&mut self) -> String {
        self.clear_rows();
        let len = self.backend.sqls.len();
        if len == 0 {
            "<<No SQL statement>>".to_string()
        } else if len > 1 {
            "<<Multiple SQL statements>>".to_string()
        } else {
            let sql = self.backend.sqls.remove(0);
            sql.to_unsafe_string()
        }
    }
    pub fn sqls_empty(&mut self) -> bool {
        self.backend.sqls.is_empty()
    }
    pub fn mock_rows(&mut self, select: impl Into<String>, rows: Vec<row::Row>) {
        self.backend.rows.insert(select.into(), rows);
    }
}

impl<'a> MockDb<'a> {
    pub fn from(cache: &'a Cache) -> MockDb<'a> {
        Self::with_context(cache, Context::default())
    }

    pub fn with_context(cache: &'a Cache, context: Context) -> MockDb<'a> {
        MockDb {
            backend: MockDbBackend {
                cache,
                context,
                sqls: Vec::new(),
                rows: HashMap::new(),
            },
        }
    }
}
