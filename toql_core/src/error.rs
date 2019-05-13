
use crate::query_parser::Rule;
use std::fmt;
use crate::sql_builder::SqlBuilderError;

use pest::error::Error as PestError;

 #[cfg(feature = "mysqldb")]
use mysql::error::Error;

#[derive(Debug)]
 pub enum ToqlError {
    NotFound,
    NotUnique,
    QueryParserError(pest::error::Error<Rule>),
    EncodingError(std::str::Utf8Error),
    MapperMissing(String),
    ValueMissing(String),
    SqlBuilderError(SqlBuilderError),
    #[cfg(feature = "mysqldb")]
    MySqlError(Error)
} 

pub type Result<T> = std::result::Result<T, ToqlError>;


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

impl From<PestError<Rule>> for ToqlError {
        fn from(err: PestError<Rule>) -> ToqlError {
        ToqlError::QueryParserError(err)
    }
}

impl fmt::Display for ToqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ToqlError::NotFound =>
                write!(f, "no result found"),
            ToqlError::NotUnique =>
                write!(f, "no unique result found "),
            ToqlError::MapperMissing(ref s) =>
                write!(f, "no mapper found for `{}`", s),
            ToqlError::ValueMissing(ref s) =>
                write!(f, "no value found for `{}`", s),
            #[cfg(feature = "mysqldb")]
            ToqlError::MySqlError (ref e) => e.fmt(f),
            ToqlError::SqlBuilderError (ref e) => e.fmt(f),
            ToqlError::EncodingError (ref e) => e.fmt(f),
            ToqlError::QueryParserError (ref e) => e.fmt(f),
        }
    }
}