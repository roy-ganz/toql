use crate::sql_arg::SqlArg;

#[derive(Debug)]
pub struct Sql(pub String, pub Vec<SqlArg>);

impl Sql {
    pub fn to_unsafe_string(&self) -> String {
        let mut quoted = false;
        // Replace every ? with param
        // Replace every $1 with param 1
        // Respect quoting incl. quoted quotes
        // If params are missing they are not replaced
        let mut params = self.1.iter();
        let mut position_parsing = false;
        let mut position = String::with_capacity(8);
        let mut unsafe_string: String = String::new();

        for c in self.0.chars() {
            match c {
                '\'' => {
                    quoted = !quoted;
                    unsafe_string.push('\'');
                }
                '?' if !quoted => {
                    match params.next() {
                        Some(p) => unsafe_string.push_str(&p.to_sql_string()),
                        None => unsafe_string.push('?'),
                    };
                }
                '$' if !quoted => {
                    position_parsing = true;
                }
                ' ' if position_parsing => {
                    let pos: Result<usize, _> = position.parse();
                    match pos {
                        Ok(pos) => {
                            let arg: Option<&SqlArg> = self.1.get(pos);
                            match arg {
                                Some(v) => unsafe_string.push_str(&v.to_sql_string()),
                                None => {
                                    unsafe_string.push('$');
                                    unsafe_string.push_str(&position);
                                    unsafe_string.push(' ');
                                }
                            }
                        }
                        _ => {
                            unsafe_string.push('$');
                            unsafe_string.push_str(&position);
                            unsafe_string.push(' ');
                        }
                    }
                    position.clear();
                    position_parsing = false;
                }
                _ if position_parsing => position.push(c),
                _ => unsafe_string.push(c),
            }
        }
        unsafe_string
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
