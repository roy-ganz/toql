use crate::error::ToqlError;
/// A result with a [`ToqlError`](enum.ToqlError.html)
pub type Result<T> = std::result::Result<T, ToqlError>;