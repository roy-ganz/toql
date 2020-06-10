
use std::collections::HashSet;
use crate::sql::Sql;

pub enum SqlOrPlaceholder {
    Sql(Sql),
    Placeholder(u16, Sql)
}

impl SqlOrPlaceholder {
   
}

pub struct SqlWithPlaceholders {
    tokens:Vec<SqlOrPlaceholder>
}
impl SqlWithPlaceholders {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    pub fn push_placeholder(&mut self, no: u16, sql: Sql) {
        
        self.tokens.push(SqlOrPlaceholder::Placeholder(no, sql));
    }
    pub fn push_sql(&mut self, sql: Sql) {
        
        self.tokens.push(SqlOrPlaceholder::Sql(sql));
    }
    pub fn push_literal(&mut self,lit:&str) {
        
        if let Some(t) = self.tokens.last_mut() {
            match t  {
                SqlOrPlaceholder::Sql(s)  => s.push_literal(lit),
                _ =>  self.tokens.push(SqlOrPlaceholder::Sql(Sql(lit.to_string(), Vec::new())))
            }
        } else {
            self.tokens.push(SqlOrPlaceholder::Sql(Sql(lit.to_string(), Vec::new())));
        }
    }


    pub fn into_sql(&self, placeholders: &HashSet<u16>) -> Sql {

        let mut sql = Sql::new();
         for token in &self.tokens {
             match token {
                 SqlOrPlaceholder::Sql(s) => sql.append(&s),
                 SqlOrPlaceholder::Placeholder(n, ph) => {
                     if placeholders.contains(&n) {
                         sql.append(&ph)
                     }
                 }
             }
         }
         sql
    }
   
   

   

}
    