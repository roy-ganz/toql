use crate::sql_builder::select_stream::Select;
use std::result::Result;

/// Trait to convert result row into structs.
/// This is implements by Toql Derive for all dervied structs with gerenic row and error.
/// The implementation of primitive types with concrete row and error type must be done by the database crate. 
pub trait FromRow<R, E> {
    
    // fn skip(i: usize) -> usize; // Not possible, because depends on  number of selected fields streaming iter
    // Read row values into struct, starting from index `i`.
    fn from_row_with_index<'a, I>(
        row: &R,
        i: &mut usize,
        iter: &mut I,
    ) -> Result<Self, E>
    where
        I: Iterator<Item = &'a Select>,
        Self: std::marker::Sized;
}
