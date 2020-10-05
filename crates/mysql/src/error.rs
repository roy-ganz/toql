use mysql::Error;
/// MySQL failed to run the SQL query. For feature `mysql`
use toql_core::error::ToqlError;

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

/// A result with a [`ToqlError`](enum.ToqlError.html)
pub type Result<T> = std::result::Result<T, ToqlMySqlError>;
