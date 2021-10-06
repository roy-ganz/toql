//! A result with a [`ToqlError`](enum.ToqlError.html)
use crate::error::ToqlError;

pub type Result<T> = std::result::Result<T, ToqlError>;
