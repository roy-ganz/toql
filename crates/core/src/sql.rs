use crate::sql_arg::SqlArg;

#[derive(Debug)]
pub struct Sql(pub String, pub Vec<SqlArg>);

impl Sql {
    pub fn to_unsafe_string(&self) -> String {
        // Replace every ? with param
        let mut params = self.1.iter();

        self.0
            .chars()
            .map(|c| {
                if c == '?' {
                    match params.next() {
                        Some(p) => p.to_sql_string(),
                        None => String::from("?"),
                    }
                } else {
                    c.to_string()
                }
            })
            .collect::<String>()
    }
    pub fn append(&mut self, sql: &Sql) {
        self.0.push_str(&sql.0);
        self.1.extend_from_slice(&sql.1);
    }
    /*  pub fn insert(&mut self, pos:SqlSplicePos, sql: Sql) {
        self.0.insert_str(pos.literal,&sql.0);
        let mut j = pos.literal;
        for s in sql.1 {
            self.1.insert(j,s);
            j += 1;
        }
    } */

    pub fn push_literal(&mut self, sql_lit: &str) {
        self.0.push_str(&sql_lit);
    }
    pub fn pop_literals(&mut self, count: u8) {
        for _ in 0..count {
            self.0.pop();
        }
    }
    pub fn new() -> Self {
        Sql(String::new(), Vec::new())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Sql {
    fn default() -> Self {
        Self::new()
    }
}
