use std::result::Result;

/// Trait to convert result row into structs.
/// This is implements by Toql Derive for all dervied structs.
pub trait FromRow<R> {
    type Error;

    // Read row values into struct, starting from index `i`.
    fn from_row_with_index<'a, I>(row: &R, i: &mut usize, iter: &I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = &'a bool>,
        Self: std::marker::Sized;
}
