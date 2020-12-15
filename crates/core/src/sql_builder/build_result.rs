//!
//! Result of SQL Builder. Use it to get SQL that can be sent to the database.

use super::select_stream::SelectStream;
use crate::{
    alias_translator::AliasTranslator, parameter_map::ParameterMap, sql_expr::resolver_error::Result,
};
use crate::{
    sql::Sql,
    sql_expr::{resolver::Resolver, SqlExpr},
};
use std::collections::HashSet;
//use crate::sql_arg::SqlArg;

/// The SQL Builder Result is created by the [SQL Builder](../sql_builder/struct.SqlBuilder.html).
pub struct BuildResult {
    /*  pub(crate) table: String,
    pub(crate) canonical_table_alias: String, */
    pub(crate) any_selected: bool,
    pub(crate) distinct: bool,
    pub(crate) table_alias: String,
    pub(crate) selection_stream: SelectStream,
    pub(crate) unmerged_paths: HashSet<String>,
    pub(crate) verb_expr: SqlExpr,
    pub(crate) select_expr: SqlExpr,
    pub(crate) from_expr: SqlExpr,
    pub(crate) join_expr: SqlExpr,
    pub(crate) where_expr: SqlExpr,
    pub(crate) order_expr: SqlExpr,
    pub(crate) selected_placeholders: HashSet<u16>,
    pub(crate) column_counter: usize,
}

impl BuildResult {
    pub fn new(verb: SqlExpr) -> Self {
        BuildResult {
            /* table,
            canonical_table_alias, */
            table_alias: String::new(),
            any_selected: false,
            distinct: false,
            unmerged_paths: HashSet::new(),
            selection_stream: SelectStream::new(),
            verb_expr: verb,
            select_expr: SqlExpr::new(),
            join_expr: SqlExpr::new(),
            from_expr: SqlExpr::new(),
            where_expr: SqlExpr::new(),
            order_expr: SqlExpr::new(),
            selected_placeholders: HashSet::new(),
            column_counter: 0,
        }
    }
    /// Returns true if no field is neither selected nor filtered.
    pub fn is_empty(&self) -> bool {
        //!self.any_selected && self.where_expr.is_empty()
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
    pub fn selected_placeholders(&self) -> &HashSet<u16> {
        &self.selected_placeholders
    }

    pub fn set_from(&mut self, table: String, canonical_alias: String) {
        self.table_alias = canonical_alias.to_owned();
        self.from_expr.push_literal(table);
        self.from_expr.push_literal(" ");
        self.from_expr.push_alias(canonical_alias);
    }

    /*   pub fn push_join(&mut self, s: &str) {
           if !self.join_sql.0.ends_with(' ') {
               self.join_sql.0.push(' ');
           }
           self.join_sql.0.push_str(s);
       }
    */
    pub fn push_join(&mut self, j: SqlExpr) {
        self.join_expr.push_separator(" ".to_string());
        self.join_expr.extend(j);
    }

    pub fn to_sql_with_modifier_and_extra(
        &self,
        aux_params: &ParameterMap,
        alias_translator: &mut AliasTranslator,
        modifier: &str,
        extra: &str,
    ) -> Result<Sql> {
        let resolver = Resolver::new()
            .with_aux_params(aux_params)
            .with_placeholders(&self.selected_placeholders);
        let verb_sql = resolver.to_sql(&self.verb_expr, alias_translator)?;
        let select_sql = resolver.to_sql(&self.select_expr, alias_translator)?;
        let from_sql = resolver.to_sql(&self.from_expr, alias_translator)?;
        let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
        let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;

        let n = select_sql.1.len() + join_sql.1.len() + where_sql.1.len();
        let mut args = Vec::with_capacity(n);

        args.extend_from_slice(&select_sql.1);
        args.extend_from_slice(&join_sql.1);
        args.extend_from_slice(&where_sql.1);

        let mut stmt = String::from(verb_sql.0);
        stmt.push(' ');

        if !modifier.is_empty() {
            stmt.push_str(modifier);
            stmt.push(' ');
        }
        stmt.push_str(&select_sql.0);

        if !self.from_expr.is_empty() {
            stmt.push_str(" FROM ");
            stmt.push_str(&from_sql.0);
        }

        if !self.join_expr.is_empty() {
            stmt.push(' ');
            let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
            stmt.push_str(&join_sql.0);
        }

        if !self.where_expr.is_empty() {
            stmt.push_str(" WHERE ");
            let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;
            stmt.push_str(&where_sql.0);
        }

        if !self.order_expr.is_empty() {
            stmt.push_str(" ORDER BY ");
            let order_sql = resolver.to_sql(&self.order_expr, alias_translator)?;
            stmt.push_str(&order_sql.0);
        }

        if !extra.is_empty() {
            stmt.push(' ');
            stmt.push_str(extra);
        }

        Ok(Sql(stmt, args))
    }

    pub fn to_sql(
        &self,
        aux_params: &ParameterMap,
        alias_translator: &mut AliasTranslator,
    ) -> Result<Sql> {
        self.to_sql_with_modifier_and_extra(aux_params, alias_translator, "", "")
    }

    fn sql_body(&self, s: &mut String, alias_translator: &mut AliasTranslator) -> Result<()> {
        let resolver = Resolver::new();

        /*   s.push_str(" FROM ");
        let from_sql = resolver.to_sql(&self.from_expr, alias_translator)?;
        s.push_str(&from_sql.0);
        /*   s.push_str(" ");
        s.push_str(&self.canonical_table_alias); // TODO translate */
        s.push_str(" "); */

        if !self.join_expr.is_empty() {
            s.push(' ');
            let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
            s.push_str(&join_sql.0);
        }

        if !self.where_expr.is_empty() {
            s.push_str(" WHERE ");
            let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;
            s.push_str(&where_sql.0);
        }

        if !self.order_expr.is_empty() {
            s.push_str(" ORDER BY ");
            let order_sql = resolver.to_sql(&self.order_expr, alias_translator)?;
            s.push_str(&order_sql.0);
        }
        Ok(())
    }

    /*  pub fn delete_sql(&self) -> Result<Sql> {

        let resolver = Resolver::new();
        let join_sql = resolver.to_sql(&self.join_expr)?;
        let where_sql = resolver.to_sql(&self.where_expr)?;

        let n=  join_sql.1.len() + where_sql.1.len() ;
        let mut args = Vec::with_capacity(n);

        args.extend_from_slice(&join_sql.1);
        args.extend_from_slice(&where_sql.1);

        let mut stmt = String::from("DELETE");
        stmt.push_str(&self.aliased_table.trim_start_matches(|c|c != ' ')); // Remove Table type to get only alias
        self.sql_body(&mut stmt);

        Ok(Sql(stmt,args))
    } */
    /*  pub fn select_sql(
        &self,
        modifier: &str,
        extra: &str,
        alias_translator: &mut AliasTranslator,
    ) -> Result<Sql> {
        self.select_sql_with_additional_columns::<&str>(alias_translator, modifier, extra, &[])
    } */

    /* pub fn select_sql_with_additional_columns<T: AsRef<str>>(
        &self,
        alias_translator: &mut AliasTranslator,
        modifier: &str,
        extra: &str,
        columns: &[T],
    ) -> Result<Sql> {
        let resolver = Resolver::new();
        let select_sql = resolver.to_sql(&self.select_expr, alias_translator)?;
        let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;
        let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;
        let order_sql = resolver.to_sql(&self.order_expr, alias_translator)?;

        let n = select_sql.1.len() + join_sql.1.len() + where_sql.1.len() + order_sql.1.len();
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&select_sql.1);
        args.extend_from_slice(&join_sql.1);
        args.extend_from_slice(&where_sql.1);
        args.extend_from_slice(&order_sql.1);

        let mut stmt = String::from("SELECT ");
        if self.distinct {
            stmt.push_str("DISTINCT ");
        }
        if !modifier.is_empty() {
            stmt.push_str(modifier);
            stmt.push(' ');
        }
        stmt.push_str(&select_sql.0);
        for c in columns {
            stmt.push_str(" ,");
            stmt.push_str(c.as_ref());
        }
        self.sql_body(&mut stmt, alias_translator)?;
        if !extra.is_empty() {
            stmt.push(' ');
            stmt.push_str(extra);
        }

        Ok(Sql(stmt, args))
    } */

    /// Returns delete SQL statement.
    /*  pub fn delete_stmt(&self) -> String {
        let mut s = String::from("DELETE");
         s.push_str(&self.aliased_table.trim_start_matches(|c|c != ' ')); // Remove Table type to get only alias
        self.sql_body(&mut s);
        s
    } */

    /// Returns count SQL statement.
    pub fn to_count_sql(&self, alias_translator: &mut AliasTranslator) -> Result<Sql> {
        let mut stmt = String::from("SELECT ");
        if self.distinct {
            stmt.push_str("COUNT(DISTINCT *)");
        } else {
            stmt.push_str("COUNT(*)");
        }

        self.sql_body(&mut stmt, alias_translator)?;

        let resolver = Resolver::new();
        let where_sql = resolver.to_sql(&self.where_expr, alias_translator)?;
        let join_sql = resolver.to_sql(&self.join_expr, alias_translator)?;

        let n = join_sql.1.len() + where_sql.1.len();
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&join_sql.1);
        args.extend_from_slice(&where_sql.1);

        Ok(Sql(stmt, args))
    }

    /// Returns simple SQL.
    /*  pub fn query_stmt(
        &self,
        mut alias_translator: &mut AliasTranslator,
        modifier: &str,
        extra: &str,
    ) -> Result<String> {
        let mut s = String::from("SELECT ");
        if self.distinct {
            s.push_str("DISTINCT ");
        }
        if !modifier.is_empty() {
            s.push_str(modifier);
            s.push(' ');
        }

        let resolver = Resolver::new();
        let select_sql = resolver.to_sql(&self.select_expr, alias_translator)?;

        s.push_str(&select_sql.0);
        self.sql_body(&mut s, alias_translator);
        if !extra.is_empty() {
            s.push(' ');
            s.push_str(extra);
        }

        Ok(s)
    } */

    /*  /// Returns SQL parameters for the WHERE and HAVING clauses in SQL.
    pub fn query_params(&self) -> Vec<SqlArg> {

        let n= self.select_expr.1.len() + self.join_expr.1.len() + self.where_expr.1.len() + self.order_expr.1.len();
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&self.select_expr.1);
        args.extend_from_slice(&self.join_expr.1);
        args.extend_from_slice(&self.where_expr.1);
        args.extend_from_slice(&self.order_expr.1);
        args

    } */

    pub fn selection_stream(&self) -> &SelectStream {
        &self.selection_stream
    }
    pub fn unmerged_paths(&self) -> &HashSet<String> {
        &self.unmerged_paths
    }

    pub fn resolve_placeholders(&mut self) {
        Self::selection_from_token(
            &self.selected_placeholders,
            &mut self.selection_stream,
            &self.select_expr,
            &mut self.column_counter,
        );
    }

    fn selection_from_token(
        selected_placeholders: &HashSet<u16>,
        select_stream: &mut SelectStream,
        expr: &SqlExpr,
        column_counter: &mut usize,
    ) {
        use super::select_stream::Select;
        use crate::sql_expr::SqlExprToken;

        for token in expr.tokens() {
            match token {
                SqlExprToken::Placeholder(number, expr, sel) => {
                    if selected_placeholders.contains(number) {
                        select_stream.change(*sel, Select::Preselect);
                        *column_counter += 1;
                    }
                    Self::selection_from_token(
                        selected_placeholders,
                        select_stream,
                        expr,
                        column_counter,
                    );
                }
                _ => {}
            }
        }
    }

    /*  pub(crate) fn push_pending_parens(clause: &mut String, pending_parens: &u8) {
        for _n in 0..*pending_parens {
            clause.push_str("(");
        }
    }
    pub(crate) fn push_concatenation(
        clause: &mut String,
        pending_concatenation: &Option<Concatenation>,
    ) {
        if let Some(c) = pending_concatenation {
            match c {
                Concatenation::And => clause.push_str(" AND "),
                Concatenation::Or => clause.push_str(" OR "),
            }
        }
    }
    pub(crate) fn push_filter(clause: &mut String, filter: &str) {
        clause.push_str(filter);
    } */
}
