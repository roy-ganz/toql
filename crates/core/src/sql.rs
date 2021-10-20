//! A raw SQL statement.

use crate::sql_arg::SqlArg;

///  A tuple to hold a raw SQL statement and the SQL arguments.
/// `Sql` is the result from the [Resolver](crate::sql_expr::resolver::Resolver)
/// and is ready to be sent to the database.
#[derive(Debug)]
pub struct Sql(pub String, pub Vec<SqlArg>);

impl Sql {
    /// Builds a string with all arguments inlined.
    ///
    /// While the string could technically be sent to a database
    /// never do this, because of the risk of SQL injection!
    /// The string is should only be used for debugging and logging purposes.
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
    /// Add `Sql` at the end.
    pub fn append(&mut self, sql: &Sql) {
        self.0.push_str(&sql.0);
        self.1.extend_from_slice(&sql.1);
    }
    /// Add a literal string at the end.
    pub fn push_literal(&mut self, sql_lit: &str) {
        self.0.push_str(&sql_lit);
    }
    /// Remove a number of characters from the end.
    pub fn pop_literals(&mut self, count: u8) {
        for _ in 0..count {
            self.0.pop();
        }
    }
    /// Create a new empty SQL statement
    pub fn new() -> Self {
        Sql(String::new(), Vec::new())
    }
    /// Returns true, if statement is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Sql {
    fn default() -> Self {
        Self::new()
    }
}
