use crate::error::ToqlError;
use crate::query::field_path::{Descendents, FieldPath};
use std::collections::HashMap;
use std::result::Result;

// R is database specific row, E the desired output error
// Trait is implemented for structs that can deserialize from rows
pub trait TreeIndex<R, E>
where
    E: std::convert::From<ToqlError>,
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
