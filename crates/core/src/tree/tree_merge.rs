use crate::{query::field_path::FieldPath, sql_builder::select_stream::SelectStream};
use std::collections::HashMap;

// R is database specific row
// Trait is implemented for structs that can deserialize from rows
pub trait TreeMerge<R, E> {
    fn merge<'a, I>(
        &mut self,
        descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &HashMap<u64, Vec<usize>>,
        selection_stream: &SelectStream,
    ) -> Result<(), E>
    where
        I: Iterator<Item = FieldPath<'a>>;
}
