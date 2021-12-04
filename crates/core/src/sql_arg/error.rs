//! Error type for TryInto.

/// Tuple struct that holds the [SqlArg](super::SqlArg) that could not be converted
/// into the desired datatype.
#[derive(Debug, PartialEq)]
pub struct TryFromSqlArgError(pub super::SqlArg);
