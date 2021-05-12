use super::Join;
use crate::error::ToqlError;
use crate::query::field_path::{Descendents, FieldPath};
use crate::sql_builder::select_stream::SelectStream;
use crate::tree::tree_merge::TreeMerge;
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
