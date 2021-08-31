use crate::{error::ToqlError, query::field_path::FieldPath};
use std::{collections::HashMap, result::Result};

// R is database specific row, E the desired output error
// Trait is implemented for structs that can deserialize from rows
pub trait TreeIndex<R, E>
{
    fn index<'a, I>(
        descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
