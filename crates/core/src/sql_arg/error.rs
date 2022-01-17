//! Error type for TryInto.

/// Tuple struct that holds the [SqlArg](super::SqlArg) that could not be converted
/// into the desired datatype.
use thiserror::Error;

#[derive(Error, Debug)]
#[error("unable to convert `{0}` into desired type")]
pub struct TryFromSqlArgError(pub super::SqlArg);
