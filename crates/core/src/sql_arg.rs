pub mod error;
pub mod from;
pub mod try_into;
pub mod from_row;

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
    pub fn to_sql_string(&self) -> String {
        match self {
            SqlArg::U64(t) => t.to_string(),
            SqlArg::I64(t) => t.to_string(),
            SqlArg::F64(t) => t.to_string(),
            SqlArg::Str(t) => format!("'{}'", t.replace("'", "''")),
            SqlArg::Bool(t) => t.to_string(),
            SqlArg::Null => "NULL".to_string(),
        }
    }

    pub fn get_i64(&self) -> Option<i64> {
        if let Self::I64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    pub fn get_f64(&self) -> Option<f64> {
        if let Self::F64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    pub fn get_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    pub fn get_u64(&self) -> Option<u64> {
        if let Self::U64(v) = self {
            Some(v.to_owned())
        } else {
            None
        }
    }
    pub fn get_str(&self) -> Option<&str> {
        if let Self::Str(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
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
            SqlArg::Str(t) => t.to_string(),
            SqlArg::Bool(t) => t.to_string(),
            SqlArg::Null => "NULL".to_string(),
        }
    }
}


pub fn is_invalid(args: &[SqlArg]) -> bool {

    args.iter().any(|a| match a {
     SqlArg::U64(x) => x == &0,
     SqlArg::Str(x) => x.is_empty(),
    _ => false    
    }
    )
}

/*
impl TryInto<Option<u32>> for &SqlArg {
    type Error = TryFromSqlArgError;
    fn try_into(self) -> Result<Option<u32>, Self::Error> {

       if self.is_null() {
           Ok(None)
       } else {
        let v =  self. get_u64().ok_or(TryFromSqlArgError(self.to_owned()))?;
        <u32 as std::convert::TryFrom<_>>::try_from(v)
        .map(|v| Some(v))
        .map_err(|_|TryFromSqlArgError(self.to_owned()))
       }
    }
}  */
