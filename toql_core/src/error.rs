
use std::fmt;
use crate::sql_builder::SqlBuilderError;

 #[cfg(feature = "mysqldb")]
use mysql::error::Error;

#[derive(Debug)]
 pub enum ToqlError {
    NotFound, 
    NotUnique,
    MapperMissing(String),
    SqlBuilderError(SqlBuilderError),

    #[cfg(feature = "mysqldb")]
    MySqlError(Error)
} 

impl From<SqlBuilderError> for ToqlError {
        fn from(err: SqlBuilderError) -> ToqlError {
        ToqlError::SqlBuilderError(err)
    }
}

#[cfg(feature = "mysqldb")]
impl From<Error> for ToqlError {
        fn from(err: Error) -> ToqlError {
        ToqlError::MySqlError(err)
    }
}

impl fmt::Display for ToqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ToqlError::NotFound =>
                write!(f, "no result found"),
            ToqlError::NotUnique =>
                write!(f, "no unique result found"),
            ToqlError::MapperMissing(ref s) =>
                write!(f, "no mapper found for `{}`", s),
            #[cfg(feature = "mysqldb")]
            ToqlError::MySqlError (ref e) => e.fmt(f),
            ToqlError::SqlBuilderError (ref e) => e.fmt(f),
        }
    }
}