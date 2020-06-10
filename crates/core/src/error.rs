//! Error handling.
//!
//! ToqlError represents all library errors and wraps errors from the Pest parser and the optional database crate.
//!

use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::sql_mapper::SqlMapperError;
use std::fmt;

use pest::error::Error as PestError;

#[macro_export]
macro_rules! ok_or_fail {
    ( $var:expr ) => {
        $var.as_ref().ok_or(toql::error::ToqlError::ValueMissing(
            stringify!($var).to_string(),
        ))
    };
}

/// Represents all errors
#[derive(Debug)]
pub enum ToqlError {
    /// No single record found for the Toql query.
    NotFound,
    /// Many records found, when exactly one was expected.
    NotUnique,
    /// The query parser encountered a syntax error.
    QueryParserError(PestError<toql_query_parser::Rule>),
    /// The sql expression parser encountered a syntax error.
    SqlExprParserError(PestError<toql_sql_expr_parser::Rule>),
    /// The query encoding was not valid UTF-8.
    EncodingError(std::str::Utf8Error),
    /// No mapper was found for a given struct. Contains the struct name.
    MapperMissing(String),
    /// The Mapper encountered an error
    SqlMapperError(SqlMapperError),
    /// Unable to put database result into struct. Contains field name.
    ValueMissing(String),
    /// SQL Builder failed to turn Toql query into SQL query.
    SqlBuilderError(SqlBuilderError),
    /// Toql failed to convert row value into struct field
    DeserializeError(String, String),
}

/// A result with a [`ToqlError`](enum.ToqlError.html)
pub type Result<T> = std::result::Result<T, ToqlError>;

impl From<SqlBuilderError> for ToqlError {
    fn from(err: SqlBuilderError) -> ToqlError {
        ToqlError::SqlBuilderError(err)
    }
}
impl From<SqlMapperError> for ToqlError {
    fn from(err: SqlMapperError) -> ToqlError {
        ToqlError::SqlMapperError(err)
    }
}

impl From<PestError<toql_query_parser::Rule>> for ToqlError {
    fn from(err: PestError<toql_query_parser::Rule>) -> ToqlError {
        ToqlError::QueryParserError(err)
    }
}
impl From<PestError<toql_sql_expr_parser::Rule>> for ToqlError {
    fn from(err: PestError<toql_sql_expr_parser::Rule>) -> ToqlError {
        ToqlError::SqlExprParserError(err)
    }
}

impl fmt::Display for ToqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ToqlError::NotFound => write!(f, "no result found"),
            ToqlError::NotUnique => write!(f, "no unique result found "),
            ToqlError::MapperMissing(ref s) => write!(f, "no mapper found for `{}`", s),
            ToqlError::SqlMapperError(ref e) => e.fmt(f),
            ToqlError::ValueMissing(ref s) => write!(f, "no value found for `{}`", s),

            ToqlError::SqlBuilderError(ref e) => e.fmt(f),
            ToqlError::EncodingError(ref e) => e.fmt(f),
            ToqlError::QueryParserError(ref e) => e.fmt(f),
            ToqlError::SqlExprParserError(ref e) => e.fmt(f),
            ToqlError::DeserializeError(ref n, ref e) => {
                write!(f, "unable to deserialize field `{}` because: {}", n, e)
            }
        }
    }
}
