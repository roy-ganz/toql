use super::Join;
use crate::{query::field_path::{ FieldPath}, sql_builder::select_stream::SelectStream, tree::tree_merge::TreeMerge, error::ToqlError};
use std::collections::HashMap;

impl<T, R, E> TreeMerge<R, E> for Join<T>
where
    T: crate::keyed::Keyed + TreeMerge<R, E>,
    E: std::convert::From<ToqlError>,
{
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
        I: Iterator<Item = FieldPath<'a>>,
    {
        match self {
            Join::Key(_) => Err(ToqlError::ValueMissing(
                descendents.next().unwrap_or_default().to_string(),
            )
            .into()),
            Join::Entity(e) => e.merge(
                descendents,
                field,
                rows,
                row_offset,
                index,
                selection_stream,
            ),
        }
    }
}
