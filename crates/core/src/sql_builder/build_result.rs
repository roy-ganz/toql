//! Result of SQL Builder. Use it to get SQL that can be sent to the database.

use super::select_stream::SelectStream;
use crate::{
    alias_translator::AliasTranslator, parameter_map::ParameterMap,
    sql_expr::resolver_error::Result,
};
use crate::{
    sql::Sql,
    sql_expr::{resolver::Resolver, SqlExpr},
};
use std::collections::HashSet;
//use crate::sql_arg::SqlArg;

/// The SQL Builder Result is created by the [SQL Builder](../sql_builder/struct.SqlBuilder.html).
pub struct BuildResult {
    pub(crate) any_selected: bool,
    pub(crate) distinct: bool,
    pub(crate) table_alias: String,
    pub(crate) select_stream: SelectStream,
    pub(crate) unmerged_home_paths: HashSet<String>,
    pub(crate) verb_expr: SqlExpr,
    pub(crate) modifier: String,
    pub(crate) preselect_expr: SqlExpr,
    pub(crate) select_expr: SqlExpr,
    pub(crate) from_expr: SqlExpr,
    pub(crate) join_expr: SqlExpr,
    pub(crate) where_expr: SqlExpr,
    pub(crate) order_expr: SqlExpr,
    pub(crate) extra: String,
    pub(crate) column_counter: usize,
}

impl BuildResult {
    /// Create a new build result
    pub fn new(verb: SqlExpr) -> Self {
        BuildResult {
            table_alias: String::new(),
            any_selected: false,
            distinct: false,
            unmerged_home_paths: HashSet::new(),
            select_stream: SelectStream::new(),
            verb_expr: verb,
            modifier: "".to_string(),
            preselect_expr: SqlExpr::new(),
            select_expr: SqlExpr::new(),
            join_expr: SqlExpr::new(),
            from_expr: SqlExpr::new(),
            where_expr: SqlExpr::new(),
            order_expr: SqlExpr::new(),
            extra: "".to_string(),
            column_counter: 0,
        }
    }
    /// Returns true if no field is neither selected nor filtered.
    pub fn is_empty(&self) -> bool {
        !self.select_expr.is_empty() && self.where_expr.is_empty()
    }
    pub fn any_selected(&self) -> bool {
        self.any_selected
    }
    pub fn push_select(&mut self, expr: SqlExpr) {
        self.select_expr.extend(expr);
    }
    pub fn table_alias(&self) -> &String {
        &self.table_alias
    }
    pub fn column_counter(&self) -> usize {
        self.column_counter
    }

    pub fn set_preselect(&mut self, preselect_expr: SqlExpr) {
        self.preselect_expr = preselect_expr;
    }
    pub fn set_modifier(&mut self, modifier: String) {
        self.modifier = modifier;
    }
    pub fn set_extra(&mut self, extra: String) {
        self.extra = extra;
    }

    pub fn set_from(&mut self, table: String, canonical_alias: String) {
        self.table_alias = canonical_alias.to_owned();
        self.from_expr.push_literal(table);
        self.from_expr.push_literal(" ");
        self.from_expr.push_alias(canonical_alias);
    }

    pub fn push_join(&mut self, j: SqlExpr) {
        self.join_expr.extend(j);
    }

    pub fn to_sql(
        &self,
        aux_params: &ParameterMap,
        alias_translator: &mut AliasTranslator,
    ) -> Result<Sql> {
        let resolver = Resolver::new().with_aux_params(aux_params);

        // Resolve compulsory parts early to estimate number of arguments
        let verb_sql = resolver.to_sql(&self.verb_expr, alias_translator)?;
        let preselect_sql = resolver.to_sql(&self.preselect_expr, alias_translator)?;
        let select_sql = resolver.to_sql(&self.select_expr, alias_translator)?;
        let from_sql = resolver.to_sql(&self.from_expr, alias_translator)?;
        let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
        let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;

        let n = preselect_sql.1.len() + select_sql.1.len() + join_sql.1.len() + where_sql.1.len();
        let mut args = Vec::with_capacity(n);

        let mut stmt = verb_sql.0;
        stmt.push(' ');

        // Optional parts
        if !self.modifier.is_empty() {
            stmt.push_str(&self.modifier);
            stmt.push(' ');
        }

        if !preselect_sql.is_empty() {
            stmt.push_str(&preselect_sql.0);
            stmt.push_str(", ");
            args.extend_from_slice(&preselect_sql.1);
        }
        stmt.push_str(&select_sql.0);
        args.extend_from_slice(&select_sql.1);

        if !self.from_expr.is_empty() {
            stmt.push_str(" FROM ");
            stmt.push_str(&from_sql.0);
            args.extend_from_slice(&from_sql.1);
        }

        if !self.join_expr.is_empty() {
            stmt.push(' ');
            let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
            stmt.push_str(&join_sql.0);
            args.extend_from_slice(&join_sql.1);
        }

        if !self.where_expr.is_empty() {
            stmt.push_str(" WHERE ");
            stmt.push_str(&where_sql.0);
            args.extend_from_slice(&where_sql.1);
        }

        if !self.order_expr.is_empty() {
            stmt.push_str(" ORDER BY ");
            let order_sql = resolver.to_sql(&self.order_expr, alias_translator)?;
            stmt.push_str(&order_sql.0);
            args.extend_from_slice(&order_sql.1);
        }
        if !self.extra.is_empty() {
            stmt.push(' ');
            stmt.push_str(&self.extra);
        }

        Ok(Sql(stmt, args))
    }

    /// Returns count SQL statement.
    pub fn set_count_select(&mut self) {
        if self.distinct {
            self.select_expr = SqlExpr::literal("COUNT(DISTINCT *)");
        } else {
            self.select_expr = SqlExpr::literal("COUNT(*)");
        }
        self.modifier = String::new();
        self.extra = String::new();
    }

    pub fn select_stream(&self) -> &SelectStream {
        &self.select_stream
    }
    pub fn unmerged_home_paths(&self) -> &HashSet<String> {
        &self.unmerged_home_paths
    }
}
