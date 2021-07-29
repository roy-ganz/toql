//! Error handling.
//!
//! ToqlError represents all library errors and wraps errors from the Pest parser and the optional database crate.
//!

use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::{
    deserialize::error::DeserializeError, sql_arg::error::TryFromSqlArgError,
    sql_expr::resolver_error::ResolverError, table_mapper::TableMapperError,
};
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
    /// Joined entity is missing, when exactly one was expected.
    JoinExpected,
    /// The query parser encountered a syntax error.
    QueryParserError(PestError<toql_query_parser::Rule>),
    /// The sql expression parser encountered a syntax error.
    SqlExprParserError(PestError<toql_sql_expr_parser::Rule>),
    /// The role expression parser encountered a syntax error.
    RoleExprParserError(PestError<toql_role_expr_parser::Rule>),
    /// The query encoding was not valid UTF-8.
    EncodingError(std::str::Utf8Error),
    /// No mapper was found for a given struct. Contains the struct name.
    MapperMissing(String),
    /// No mapper was found for a given struct. Contains the struct name.
    TryFromSqlArgError(TryFromSqlArgError),
    /// The Mapper encountered an error
    TableMapperError(TableMapperError),
    /// Unable to put database result into struct. Contains field name.
    ValueMissing(String),
    /// SQL Builder failed to turn Toql query into SQL query.
    SqlBuilderError(SqlBuilderError),
    /// Toql failed to convert row value into struct field
    DeserializeError(DeserializeError),

    /// SQL Builder failed to turn Toql query into SQL query.
    SqlExprResolverError(ResolverError),

    /// Access to shared registry, typically inside cache, failed
    RegistryPoisenError(String),

    /// Expected a value in Option<T>, but found none. Includes position. 
    /// TODO:: Check to replace with std::option::NoneError + Backtrace
    NoneError(String),
}

impl From<SqlBuilderError> for ToqlError {
    fn from(err: SqlBuilderError) -> ToqlError {
        ToqlError::SqlBuilderError(err)
    }
}

impl From<DeserializeError> for ToqlError {
    fn from(err: DeserializeError) -> ToqlError {
        ToqlError::DeserializeError(err)
    }
}

impl From<ResolverError> for ToqlError {
    fn from(err: ResolverError) -> ToqlError {
        ToqlError::SqlExprResolverError(err)
    }
}

impl From<TableMapperError> for ToqlError {
    fn from(err: TableMapperError) -> ToqlError {
        ToqlError::TableMapperError(err)
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
impl From<PestError<toql_role_expr_parser::Rule>> for ToqlError {
    fn from(err: PestError<toql_role_expr_parser::Rule>) -> ToqlError {
        ToqlError::RoleExprParserError(err)
    }
}
impl From<TryFromSqlArgError> for ToqlError {
    fn from(err: TryFromSqlArgError) -> ToqlError {
        ToqlError::TryFromSqlArgError(err)
    }
}
impl<PE> From<std::sync::PoisonError<PE>> for ToqlError {
    fn from(err: std::sync::PoisonError<PE>) -> ToqlError {
        ToqlError::RegistryPoisenError(err.to_string())
    }
}

impl fmt::Display for ToqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ToqlError::NotFound => write!(f, "no result found"),
            ToqlError::NotUnique => write!(f, "no unique result found"),
            ToqlError::JoinExpected => write!(f, "no joined value found, but expected one"),
            ToqlError::MapperMissing(ref s) => write!(f, "no mapper found for `{}`", s),
            ToqlError::TableMapperError(ref e) => e.fmt(f),
            ToqlError::ValueMissing(ref s) => write!(f, "no value found for `{}`", s),
            ToqlError::TryFromSqlArgError(ref a) => write!(
                f,
                "unable to convert `{}` into desired type",
                a.0.to_string()
            ),
            ToqlError::RegistryPoisenError(ref a) => {
                write!(f, "failed to access registry: `{}`", a.to_string())
            }
            ToqlError::SqlBuilderError(ref e) => e.fmt(f),
            ToqlError::EncodingError(ref e) => e.fmt(f),
            ToqlError::QueryParserError(ref e) => e.fmt(f),
            ToqlError::SqlExprParserError(ref e) => e.fmt(f),
            ToqlError::RoleExprParserError(ref e) => e.fmt(f),
            ToqlError::SqlExprResolverError(ref e) => e.fmt(f),
            ToqlError::DeserializeError(ref e) => e.fmt(f),
            ToqlError::NoneError(ref e) => e.fmt(f),
        }
    }
}
