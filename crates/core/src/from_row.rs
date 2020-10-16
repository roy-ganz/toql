use crate::sql_builder::select_stream::Select;
use std::result::Result;

/// Trait to convert result row into structs.
/// This is implements by Toql Derive for all dervied structs.
pub trait FromRow<R> {
    type Error;

    // fn skip(i: usize) -> usize; // Not possible, because depends on  number of selected fields streaming iter
    // Read row values into struct, starting from index `i`.
    fn from_row_with_index<'a, I>(
        row: &R,
        i: &mut usize,
        iter: &mut I,
    ) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = &'a Select>,
        Self: std::marker::Sized;
}
