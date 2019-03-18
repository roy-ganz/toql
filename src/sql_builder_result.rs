
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
    pub fn sql_for_table(&self, table: &str) -> String {
        format!(
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
        .to_string()
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