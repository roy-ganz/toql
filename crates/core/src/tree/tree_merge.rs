//! Trait to merge database rows into entities.
use crate::{query::field_path::FieldPath, sql_builder::select_stream::SelectStream};
use std::collections::HashMap;


/// The trait allows to merge rows into nested structs.
///
/// Trait is implemented by The Toql derive for structs that can deserialize from rows.
/// R is database specific row.
pub trait TreeMerge<R, E> {
    fn merge<'a, I>(
        &mut self,
        descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &HashMap<u64, Vec<usize>>,
        selection_stream: &SelectStream,
    ) -> Result<(), E>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone;
}
