//! Trait to index database rows for faster merging.
use crate::query::field_path::FieldPath;
use std::{collections::HashMap, result::Result};

/// The trait indexes all rows for nested structs.
/// It deserializes the entity key from the row array and saves its array index.
///
/// The trait is implemented by the Toql derive for structs that can be deserialize from rows
/// R is a database row type , E the database error
pub trait TreeIndex<R, E> {
    fn index<'a, I>(
        descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone;
}
