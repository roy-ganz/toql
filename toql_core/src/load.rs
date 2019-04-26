
use std::fmt;
use crate::sql_builder::SqlBuilderError;

 #[cfg(feature = "mysqldb")]
use mysql::error::Error;

#[derive(Debug)]
 pub enum LoadError {
    NotFound, 
    NotUnique,
    MapperMissing(String),
    SqlBuilderError(SqlBuilderError),

    #[cfg(feature = "mysqldb")]
    MySqlError(Error)
} 

impl From<SqlBuilderError> for LoadError {
        fn from(err: SqlBuilderError) -> LoadError {
        LoadError::SqlBuilderError(err)
    }
}

#[cfg(feature = "mysqldb")]
impl From<Error> for LoadError {
        fn from(err: Error) -> LoadError {
        LoadError::MySqlError(err)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::NotFound =>
                write!(f, "no result found"),
            LoadError::NotUnique =>
                write!(f, "no unique result found"),
            LoadError::MapperMissing(ref s) =>
                write!(f, "no mapper found for `{}`", s),
            #[cfg(feature = "mysqldb")]
            LoadError::MySqlError (ref e) => e.fmt(f),
            LoadError::SqlBuilderError (ref e) => e.fmt(f),
        }
    }
}