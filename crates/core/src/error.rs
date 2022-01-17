//! Error handling.
//!
//! ToqlError represents all library errors and wraps errors from the Pest parser and the optional database crate.
//!

use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::{
    deserialize::error::DeserializeError, sql_arg::error::TryFromSqlArgError,
    sql_expr::resolver_error::ResolverError, table_mapper::error::TableMapperError,
};
use pest::error::Error as PestError;
use thiserror::Error;

#[macro_export]
macro_rules! ok_or_fail {
    ( $var:expr ) => {
        $var.as_ref().ok_or(toql::error::ToqlError::ValueMissing(
            stringify!($var).to_string(),
        ))
    };
}

/// Represents all errors
#[derive(Error, Debug)]
pub enum ToqlError {
    /// No single record found for the Toql query.
    #[error("no result found")]
    NotFound,

    /// Many records found, when exactly one was expected.
    #[error("no unique result found")]
    NotUnique,

    /// Joined entity is missing, when exactly one was expected.
    #[error("no joined value found, but expected one")]
    JoinExpected,

    /// The query parser encountered a syntax error.
    #[error("{0}")]
    QueryParserError(#[from] PestError<toql_query_parser::Rule>),
    
    /// The sql expression parser encountered a syntax error.
    #[error("{0}")]
    SqlExprParserError(#[from] PestError<toql_sql_expr_parser::Rule>),

    /// The role expression parser encountered a syntax error.
    #[error("{0}")]
    RoleExprParserError(#[from] PestError<toql_role_expr_parser::Rule>),

    /// The query encoding was not valid UTF-8.
    #[error("{0}")]
    EncodingError(#[from] std::str::Utf8Error),

    /// No mapper was found for a given struct. Contains the struct name.
    #[error("no mapper found for `{0}`")]
    MapperMissing(String),

    /// No mapper was found for a given struct. Contains the struct name.
    #[error("{0}")]
    TryFromSqlArgError(#[from] TryFromSqlArgError),

    /// The Mapper encountered an error
    #[error("{0}")]
    TableMapperError(#[from] TableMapperError),

    /// Unable to put database result into struct. Contains field name.
    #[error("no value found for `{0}`")]
    ValueMissing(String),

    /// SQL Builder failed to turn Toql query into SQL query.
    #[error("{0}")]
    SqlBuilderError(#[from] SqlBuilderError),

    /// Toql failed to convert row value into struct field
    #[error("{0}")]
    DeserializeError(#[from] DeserializeError),

    /// SQL Builder failed to turn Toql query into SQL query.
    #[error("{0}")]
    SqlExprResolverError(#[from] ResolverError),

    /// Access to shared registry, typically inside cache, failed
    #[error("failed to access registry: `{0}`")]
    RegistryPoisenError(String),

    /// Expected a value in Option<T>, but found none. Includes position.
    #[error("{0}")]
    /// TODO:: Check to replace with std::option::NoneError + Backtrace
    NoneError(String),
}

// Manually convert to avoid generic parameter in ToqlError
impl<PE> From<std::sync::PoisonError<PE>> for ToqlError {
    fn from(err: std::sync::PoisonError<PE>) -> ToqlError {
        ToqlError::RegistryPoisenError(err.to_string())
    }
}