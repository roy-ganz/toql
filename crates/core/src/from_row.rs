//! Trait to convert result row into structs.
use crate::sql_builder::select_stream::Select;
use std::result::Result;

/// This is implemented by Toql Derive for all dervied structs with generic row and error.
/// The implementation of primitive types with concrete row and error type must be done by the database crate.
pub trait FromRow<R, E> {
    /// Returns the number of selects
    /// This is needed to advance the iterator and
    /// the row index.
    /// The Deserializer needs this information to skip left joins
    /// that have fields selected but are null.
    /// Those left joins cause select information in the select stream
    /// that must be skipped.
    fn forward<'a, I>(iter: &mut I) -> Result<usize, E>
    where
        I: Iterator<Item = &'a Select>,
        Self: std::marker::Sized;

    /// Read row values into struct, starting from index.
    /// Advances iter and index
    /// Returns None for value unselected values or joined entities that have null keys.
    /// Return Error, if value is selected, but cannot be converted.

    fn from_row<'a, I>(row: &R, index: &mut usize, iter: &mut I) -> Result<Option<Self>, E>
    where
        I: Iterator<Item = &'a Select> + Clone,
        Self: std::marker::Sized;
}
