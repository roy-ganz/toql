//!
//! Result of SQL Builder. Use it to get SQL that can be sent to the database.

use crate::query::concatenation::Concatenation;
use crate::sql::SqlArg;

/// The SQL Builder Result is created by the [SQL Builder](../sql_builder/struct.SqlBuilder.html).
pub struct BuildResult<'a> {
    pub(crate) aliased_table: &'a str,
    pub(crate) any_selected: bool,
    pub(crate) distinct: bool,
    pub(crate) join_clause: String,
    pub(crate) select_clause: String,
    pub(crate) where_clause: String,
    pub(crate) order_clause: String,
    pub(crate) having_clause: String,
    pub(crate) select_params: Vec<SqlArg>,
    pub(crate) where_params: Vec<SqlArg>,
    pub(crate) having_params: Vec<SqlArg>,
    pub(crate) order_params: Vec<SqlArg>,
    pub(crate) join_params: Vec<SqlArg>, // Not sure if needed
    pub(crate) combined_params: Vec<SqlArg>,
    
}

impl<'a> BuildResult<'a> {

    pub fn new(aliased_table: &'a str) -> Self {
    BuildResult {
            aliased_table,
            any_selected: false,
            distinct: false,
            join_clause: String::from(""),
            select_clause: String::from(""),
            where_clause: String::from(""),
            order_clause: String::from(""),
            having_clause: String::from(""),
            select_params: vec![], // query parameters in select clause, due to sql expr with <param>
            join_params: vec![],
            where_params: vec![],
            having_params: vec![],
            order_params: vec![],
            combined_params: vec![],
        }
   }
    /// Returns true if no field is neither selected nor filtered.
    /* pub fn is_empty(&self) -> bool {
        !self.any_selected && self.where_clause.is_empty() && self.having_clause.is_empty()
    } */
    pub fn any_selected(&self) -> bool {
        self.any_selected 
    }
    pub fn push_select(&mut self, s: &str) {
        if !self.select_clause.trim_end().ends_with(',') {
            self.select_clause.push(',');
        }
        self.select_clause.push_str(s);
    }
    pub fn push_join(&mut self, s: &str) {
        if !self.join_clause.ends_with(' ') {
            self.join_clause.push(' ');
        }
        self.join_clause.push_str(s);
    }  
    

    fn sql_body(&self, s: &mut String) {
       
        s.push_str(" FROM ");
        s.push_str(&self.aliased_table);
        if !self.join_clause.is_empty() {
            s.push(' ');
            s.push_str(&self.join_clause);
        }
        if !self.where_clause.is_empty() {
            s.push_str(" WHERE ");
            s.push_str(&self.where_clause);
        }
        if !self.having_clause.is_empty() {
            s.push_str(" HAVING ");
            s.push_str(&self.having_clause);
        }
        if !self.order_clause.is_empty() {
            s.push_str(" ORDER BY ");
            s.push_str(&self.order_clause);
        }
    }
  
/// Returns delete SQL statement.
    pub fn delete_stmt(&self) -> String {
        let mut s = String::from("DELETE");
         s.push_str(&self.aliased_table.trim_start_matches(|c|c != ' ')); // Remove Table type to get only alias
        self.sql_body(&mut s);
        s
    }

 /// Returns count SQL statement.
    pub fn count_stmt(&self) -> String {
        let mut s = String::from("SELECT ");
           if self.distinct {
            s.push_str("COUNT(DISTINCT *)");
        } else {
            s.push_str("COUNT(*)");
        }
       
        self.sql_body(&mut s);
       
        s
    }

    /// Returns simple SQL.
    pub fn query_stmt(&self, modifier:&str, extra:&str) -> String {
        let mut s = String::from("SELECT ");
         if self.distinct {
            s.push_str("DISTINCT ");
        }
        if !modifier.is_empty() {
            s.push_str(modifier);
            s.push(' ');
        }
        s.push_str(&self.select_clause);
        self.sql_body(&mut s);
        if !extra.is_empty() {
              s.push(' ');
            s.push_str(extra);
        }
        
        s
    }
    
    pub fn count_params(&self) -> &Vec<SqlArg> {
        &self.combined_params // TODO

        /* if self.where_params.is_empty() {
            &self.having_params
        } else if self.having_params.is_empty() {
            &self.where_params
        } else {
            &self.combined_params
        } */
    }
    /// Returns SQL parameters for the WHERE and HAVING clauses in SQL.
    pub fn query_params(&self) -> &Vec<SqlArg> {
        &self.combined_params

        /* if self.where_params.is_empty() {
            &self.having_params
        } else if self.having_params.is_empty() {
            &self.where_params
        } else {
            &self.combined_params
        } */
    }

    pub(crate) fn push_pending_parens(clause: &mut String, pending_parens: &u8) {
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
    }
}
