
//! Error handling.
//!
//! ToqlError represents all library errors and wraps errors from the Pest parser and the optional database crate.
//!
use crate::query_parser::Rule;
use std::fmt;
use crate::sql_builder::SqlBuilderError;

use pest::error::Error as PestError;

 #[cfg(feature = "mysqldb")]
use mysql::error::Error;

/// Represents all errors
#[derive(Debug)]
 pub enum ToqlError {
    /// No single record found for the Toql query.
    NotFound,
    /// Many records found, while exactly one was expected.
    NotUnique,
    /// The query parser encountered a syntax error.
    QueryParserError(pest::error::Error<Rule>),
    /// The query encoding was not valid UTF-8.
    EncodingError(std::str::Utf8Error),
    /// No mapper was found for a given struct. Contains the struct name.
    MapperMissing(String),
    /// TODO unclear
    ValueMissing(String),
    /// SQL Builder failed to turn Toql query into SQL query.
    SqlBuilderError(SqlBuilderError),
    #[cfg(feature = "mysqldb")]
    /// MySQL failed to run the SQL query. For feature `mysql`
    MySqlError(Error)
} 

/// A result with a [`ToqlError`](struct.ToqlError.html)
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