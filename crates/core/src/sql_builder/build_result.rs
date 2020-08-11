//!
//! Result of SQL Builder. Use it to get SQL that can be sent to the database.

use std::collections::HashSet;
use crate::query::concatenation::Concatenation;
use crate::sql::Sql;
use crate::sql_arg::SqlArg;


/// The SQL Builder Result is created by the [SQL Builder](../sql_builder/struct.SqlBuilder.html).
pub struct BuildResult {
    pub(crate) aliased_table: String,
    pub(crate) any_selected: bool,
    pub(crate) distinct: bool,

    pub (crate) selection_stream: Vec<bool>,
    pub (crate) unmerged_paths: HashSet<String>,
    pub (crate) select_sql: Sql,
    pub (crate) join_sql: Sql,
    pub (crate) where_sql: Sql,
    pub (crate) order_sql: Sql,
    
    
/* 
    pub(crate) join_clause: String,
    pub(crate) join_params: Vec<SqlArg>, // Not sure if needed 

    pub(crate) select_clause: String,
    pub(crate) where_clause: String,
    pub(crate) order_clause: String,
    pub(crate) select_params: Vec<SqlArg>,
    pub(crate) where_params: Vec<SqlArg>,
    pub(crate) order_params: Vec<SqlArg>,
    */
   

    
}

impl BuildResult {

    pub fn new(aliased_table: String) -> Self {
    BuildResult {
            aliased_table,
            any_selected: false,
            distinct: false,
            unmerged_paths: HashSet::new(),
            selection_stream: Vec::new(),
            select_sql: Sql::new(),
            join_sql:Sql::new(),
            where_sql:Sql::new(),
            order_sql:Sql::new(),
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
         if !self.select_sql.0.trim_end().ends_with(',') {
            self.select_sql.0.push(',');
        } 
        self.select_sql.push_literal(s);
    }
    pub fn push_join(&mut self, s: &str) {
        if !self.join_sql.0.ends_with(' ') {
            self.join_sql.0.push(' ');
        }
        self.join_sql.0.push_str(s);
    }  
    

    fn sql_body(&self, s: &mut String) {
       
        s.push_str(" FROM ");
        s.push_str(&self.aliased_table);
        if !self.join_sql.is_empty() {
            s.push(' ');
            s.push_str(&self.join_sql.0);
        }
        if !self.where_sql.is_empty() {
            s.push_str(" WHERE ");
            s.push_str(&self.where_sql.0);
        }
       
        if !self.order_sql.is_empty() {
            s.push_str(" ORDER BY ");
            s.push_str(&self.order_sql.0);
        }
    }
  
    pub fn delete_sql(&self) -> Sql {
        let n=  self.join_sql.1.len() + self.where_sql.1.len() ;
        let mut args = Vec::with_capacity(n);
      
        args.extend_from_slice(&self.join_sql.1);
        args.extend_from_slice(&self.where_sql.1);

        let mut stmt = String::from("DELETE");
        stmt.push_str(&self.aliased_table.trim_start_matches(|c|c != ' ')); // Remove Table type to get only alias
        self.sql_body(&mut stmt);
      
        Sql(stmt,args)
    }
 pub fn select_sql(&self, modifier: &str, extra: &str)  -> Sql {
     self.select_sql_with_additional_columns::<&str>(modifier, extra, &[])
 }

 pub fn select_sql_with_additional_columns<T: AsRef<str>>(&self, modifier: &str, extra: &str, columns: &[T]) -> Sql {
       let n=    self.select_sql.1.len() + self.join_sql.1.len() 
            + self.where_sql.1.len() + self.order_sql.1.len() ;
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&self.select_sql.1);
        args.extend_from_slice(&self.join_sql.1);
        args.extend_from_slice(&self.where_sql.1);
        args.extend_from_slice(&self.order_sql.1);

        let mut stmt = String::from("SELECT ");
        if self.distinct {
            stmt.push_str("DISTINCT ");
        }
        if !modifier.is_empty() {
            stmt.push_str(modifier);
            stmt.push(' ');
        }
        stmt.push_str(&self.select_sql.0);
        for c in columns {
            stmt.push_str(" ,");
            stmt.push_str(c.as_ref());
        }
        self.sql_body(&mut stmt);
        if !extra.is_empty() {
              stmt.push(' ');
            stmt.push_str(extra);
        }
      
        Sql(stmt,args)
    }

/// Returns delete SQL statement.
   /*  pub fn delete_stmt(&self) -> String {
        let mut s = String::from("DELETE");
         s.push_str(&self.aliased_table.trim_start_matches(|c|c != ' ')); // Remove Table type to get only alias
        self.sql_body(&mut s);
        s
    } */

 /// Returns count SQL statement.
    pub fn count_sql(&self) -> Sql {
        let mut stmt = String::from("SELECT ");
           if self.distinct {
            stmt.push_str("COUNT(DISTINCT *)");
        } else {
            stmt.push_str("COUNT(*)");
        }
       
        self.sql_body(&mut stmt);
       
        let n =  self.join_sql.1.len() + self.where_sql.1.len();
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&self.join_sql.1);
        args.extend_from_slice(&self.where_sql.1);
        
        Sql(stmt, args)
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
        s.push_str(&self.select_sql.0);
        self.sql_body(&mut s);
        if !extra.is_empty() {
              s.push(' ');
            s.push_str(extra);
        }
        
        s
    }
    
   
    /// Returns SQL parameters for the WHERE and HAVING clauses in SQL.
    pub fn query_params(&self) -> Vec<SqlArg> {
       
        let n= self.select_sql.1.len() + self.join_sql.1.len() + self.where_sql.1.len() + self.order_sql.1.len();
        let mut args = Vec::with_capacity(n);
        args.extend_from_slice(&self.select_sql.1);
        args.extend_from_slice(&self.join_sql.1);
        args.extend_from_slice(&self.where_sql.1);
        args.extend_from_slice(&self.order_sql.1);
        args
       
    }

    pub fn selection_stream(&self) -> &Vec<bool> {
        &self.selection_stream
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

