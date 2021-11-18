//! An argument for SQL expressions.

pub mod error;
pub mod from;
pub mod from_row;
pub mod try_into;

/// Enum to keep the different argument types.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlArg {
    U64(u64),
    I64(i64),
    F64(f64),
    Str(String),
    Bool(bool),
    Null,
}

impl SqlArg {
    /// Build SQL string.
    pub fn to_sql_string(&self) -> String {
        match self {
            SqlArg::U64(t) => t.to_string(),
            SqlArg::I64(t) => t.to_string(),
            SqlArg::F64(t) => t.to_string(),
            SqlArg::Str(t) => format!("'{}'", t.replace("'", "''")),
            SqlArg::Bool(t) => String::from(if *t { "TRUE" } else { "FALSE" }),
            SqlArg::Null => "NULL".to_string(),
        }
    }

    /// Return i64 or None, if type mismatches.
    pub fn get_i64(&self) -> Option<i64> {
        if let Self::I64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    /// Return f64 or None, if type mismatches.
    pub fn get_f64(&self) -> Option<f64> {
        if let Self::F64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    /// Return bool or None, if type mismatches.
    pub fn get_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    /// Return u64 or None, if type mismatches.
    pub fn get_u64(&self) -> Option<u64> {
        if let Self::U64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    /// Return str or None, if type mismatches.
    pub fn get_str(&self) -> Option<&str> {
        if let Self::Str(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true, if argument is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    /// Returns true, if argument is string and matches other string.
    pub fn cmp_str(&self, other: &str) -> bool {
        if let Self::Str(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl ToString for SqlArg {
    fn to_string(&self) -> String {
        match self {
            SqlArg::U64(t) => t.to_string(),
            SqlArg::I64(t) => t.to_string(),
            SqlArg::F64(t) => t.to_string(),
            SqlArg::Str(t) => format!("'{}'", t.to_string()),
            SqlArg::Bool(t) => format!("{}", (if *t { 1 } else { 0 })),
            SqlArg::Null => "0".to_string(),
        }
    }
}

/// Returns true, if list of arguments would be a valid key.
pub fn valid_key(args: &[SqlArg]) -> bool {
    let contains_zero_key = args.iter().any(|a| match a {
        SqlArg::U64(x) => x == &0,
        SqlArg::I64(x) => x == &0,
        SqlArg::Str(x) => x.is_empty(),
        SqlArg::Null => true,
        _ => false,
    });
    !contains_zero_key
}

#[cfg(test)]
mod test {
    use super::{valid_key, SqlArg};

    #[test]
    fn convert_u64() {
        let a = SqlArg::from(1u64);
        let x = a.get_u64().unwrap();
        assert_eq!(x, 1u64);
        assert_eq!(a.to_string(), "1");
        assert_eq!(a.to_sql_string(), "1");
        assert_eq!(a.cmp_str("1"), false);
    }
    #[test]
    fn convert_i64() {
        let a = SqlArg::from(1i64);
        let x = a.get_i64().unwrap();
        assert_eq!(x, 1i64);
        assert_eq!(a.to_string(), "1");
        assert_eq!(a.to_sql_string(), "1");
        assert_eq!(a.cmp_str("1"), false);
    }
    #[test]
    fn convert_f64() {
        let a = SqlArg::from(1.0f64);
        let x = a.get_f64().unwrap();
        assert_eq!(x, 1f64);
        assert_eq!(a.to_string(), "1");
        assert_eq!(a.to_sql_string(), "1");
        assert_eq!(a.cmp_str("1"), false);
    }
    #[test]
    fn convert_str() {
        let a = SqlArg::from("1");
        let x = a.get_str().unwrap();
        assert_eq!(x, "1");
        assert_eq!(a.to_string(), "'1'");
        assert_eq!(a.to_sql_string(), "'1'");
        assert_eq!(a.cmp_str("1"), true);
    }
    #[test]
    fn convert_bool() {
        let a = SqlArg::from(true);
        let x = a.get_bool().unwrap();
        assert_eq!(x, true);
        assert_eq!(a.to_string(), "1");
        assert_eq!(a.to_sql_string(), "TRUE");
        assert_eq!(a.cmp_str("true"), false);

        let a = SqlArg::from(&true);
        let x = a.get_bool().unwrap();
        assert_eq!(x, true);

        let a = SqlArg::from(Some(true));
        let x = a.get_bool().unwrap();
        assert_eq!(x, true);
    }
    #[test]
    fn convert_null() {
        let a = SqlArg::Null;
        assert_eq!(a.is_null(), true);
        assert_eq!(a.to_string(), "0");
        assert_eq!(a.to_sql_string(), "NULL");
        assert_eq!(a.cmp_str("null"), false);

        assert_eq!(a.get_u64().is_none(), true);
        assert_eq!(a.get_i64().is_none(), true);
        assert_eq!(a.get_f64().is_none(), true);
        assert_eq!(a.get_str().is_none(), true);
        assert_eq!(a.get_bool().is_none(), true);
    }
    #[test]
    fn valid_keys() {
        // Good example
        let a = [SqlArg::from(1), SqlArg::from("2")];
        assert_eq!(valid_key(&a), true);

        // Zero value is invalid
        let a = [SqlArg::from(1), SqlArg::from(0)];
        assert_eq!(valid_key(&a), false);

        // Empty string is invalid (equals to 0 in SQL)
        let a = [SqlArg::from(1), SqlArg::from("")];
        assert_eq!(valid_key(&a), false);

        // NULL value is invalid (primary keys must not be nullable)
        let a = [SqlArg::from(1), SqlArg::Null];
        assert_eq!(valid_key(&a), false);

        // Float value is valid (but bad idea db design)
        let a = [SqlArg::from(1), SqlArg::from(1f64)];
        assert_eq!(valid_key(&a), true);

        // Boolean value is valid
        let a = [SqlArg::from(1), SqlArg::from(false)];
        assert_eq!(valid_key(&a), true);
    }
}
