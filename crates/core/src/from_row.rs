use crate::{sql_builder::select_stream::Select};
use std::result::Result;

/// Trait to convert result row into structs.
/// This is implements by Toql Derive for all dervied structs with gerenic row and error.
/// The implementation of primitive types with concrete row and error type must be done by the database crate. 
pub trait FromRow<R, E> 
{
    
    
    /// Read row values into struct, starting from index.
    /// Advances iter and index
    /// Returns None, if value is not selected. 
    /// Return Error, if value is selected, but cannot be converted.
    
    
    fn from_row<'a, I>(
        row: &R,
        index: &mut usize,
        iter: &mut I,
    ) -> Result<Option<Self>, E>
    where
        I: Iterator<Item = &'a Select>,
        Self: std::marker::Sized;
}
