use thiserror::Error;

/// Error that may happen when a database row is deseriazed
#[derive(Error, Debug)]
pub enum DeserializeError {
    /// A selection is expected, but the [SelectStream](crate::sql_builder::select_stream::SelectStream) is `None`.
    #[error("expected field `{0}` to be selected, but got unselected")]
    SelectionExpected(String),

    /// The [SelectStream](crate::sql_builder::select_stream::SelectStream) ended unexpectedly.
    #[error("expected stream to contain more selections")]
    StreamEnd,

    /// Conversion from database row into Rust struct field failed.
    #[error("unable to convert field `{0}`, because `{1}`")]
    ConversionFailed(String, String),
}
