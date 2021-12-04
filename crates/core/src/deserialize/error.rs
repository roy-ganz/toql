use std::fmt;

/// Error that may happen when a database row is deseriazed
#[derive(Debug, PartialEq)]
pub enum DeserializeError {
    /// A selection is expected, but the [SelectStream](crate::sql_builder::select_stream::SelectStream) is `None`.
    SelectionExpected(String),
    /// The [SelectStream](crate::sql_builder::select_stream::SelectStream) ended unexpectedly.
    StreamEnd,
    /// Conversion from database row into Rust struct field failed.
    ConversionFailed(String, String),
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::SelectionExpected(ref field) => write!(
                f,
                "expected field `{}` to be selected, but got unselected.",
                field
            ),
            DeserializeError::StreamEnd => write!(f, "expected stream to contain more selections"),
            DeserializeError::ConversionFailed(ref field, ref cause) => write!(
                f,
                "unable to convert field `{}`, because `{}`",
                field, cause
            ),
        }
    }
}
