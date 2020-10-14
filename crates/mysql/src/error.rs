use mysql::Error;
/// MySQL failed to run the SQL query. For feature `mysql`
use toql_core::{sql_builder::sql_builder_error::SqlBuilderError, error::ToqlError};

#[derive(Debug)]
pub enum ToqlMySqlError {
    ToqlError(ToqlError),
    MySqlError(Error),
}

impl From<Error> for ToqlMySqlError {
    fn from(err: Error) -> ToqlMySqlError {
        ToqlMySqlError::MySqlError(err)
    }
}
impl From<ToqlError> for ToqlMySqlError {
    fn from(err: ToqlError) -> ToqlMySqlError {
        ToqlMySqlError::ToqlError(err)
    }
}
impl From<SqlBuilderError> for ToqlMySqlError {
    fn from(err: SqlBuilderError) -> ToqlMySqlError {
        ToqlMySqlError::ToqlError(err.into())
    }
}

/// A result with a [`ToqlError`](enum.ToqlError.html)
pub type Result<T> = std::result::Result<T, ToqlMySqlError>;
