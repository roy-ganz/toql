
use crate::query::Concatenation;



pub struct SqlBuilderResult {
    pub join_clause: String,
    pub select_clause: String,
    pub where_clause: String,
    pub order_by_clause: String,
    pub having_clause: String,
    pub count_where_clause: String,
    pub count_having_clause: String,

    pub where_params: Vec<String>,
    pub having_params: Vec<String>,
}

impl SqlBuilderResult {

    fn sql_body_for_table(&self,table: &str,  s: &mut String)  {
        s.push_str(&self.select_clause);
        s.push_str(" FROM ");
        s.push_str(table);
         if !self.join_clause.is_empty() {
            s.push(' ');
            s.push_str(&self.join_clause);
         }
         if !self.where_clause.is_empty() {
            s.push_str(" WHERE " );
            s.push_str(&self.where_clause);
         }
         if !self.having_clause.is_empty() {
           s.push_str(" HAVING " );
            s.push_str(&self.having_clause);
         }
         if !self.order_by_clause.is_empty() {
            s.push_str(" ORDER BY " );
            s.push_str(&self.order_by_clause);
         }
         
    }
    // Put behind feature

    pub fn sql_for_mysql_table(&self, table: &str, hint:&str, offset:u64, max: u16) -> String {

        let mut s = String::from("SELECT ");

        if !hint.is_empty() {
            s.push_str(hint);
              s.push(' ');
        }
         self.sql_body_for_table(table, &mut s);
         s.push_str(" LIMIT ");
         s.push_str(&offset.to_string());
         s.push(',');
         s.push_str(&max.to_string());

         s
    }

    pub fn sql_for_table(&self, table: &str) -> String {

          let mut s = String::from("SELECT ");
           self.sql_body_for_table(table, &mut s);
           s
        /* format!(
            "SELECT {} FROM {}{}{}{}{}",
            self.select_clause,
            table,
             if self.join_clause.is_empty() {
                String::from("")
            } else {
                format!(" {}", self.join_clause)
            },
            if self.where_clause.is_empty() {
                String::from("")
            } else {
                format!(" WHERE {}", self.where_clause)
            },
            if self.having_clause.is_empty() {
                String::from("")
            } else {
                format!(" HAVING {}", self.having_clause)
            },
            if self.order_by_clause.is_empty() {
                String::from("")
            } else {
                format!(" ORDER BY {}", self.order_by_clause)
            }
        )
        .trim_end()
        .to_string() */
    }

    pub (crate) fn push_pending_parens(clause: &mut String, pending_parens: &u8) {
        for _n in 0..*pending_parens {
            clause.push_str("(");
        }
    }
    pub (crate) fn push_concatenation(clause: &mut String, pending_concatenation: &Option<Concatenation>) {
        if let Some(c) = pending_concatenation {
            match c {
                Concatenation::And => clause.push_str(" AND "),
                Concatenation::Or => clause.push_str(" OR "),
            }
        }
    }
    pub (crate) fn push_filter(clause: &mut String, filter: &str) {
        clause.push_str(filter);
    }
}