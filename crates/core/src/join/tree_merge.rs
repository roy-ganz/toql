use super::Join;
use crate::{
    error::ToqlError, query::field_path::FieldPath, sql_builder::select_stream::SelectStream,
    tree::tree_merge::TreeMerge,
};
use std::collections::HashMap;

impl<T, R, E> TreeMerge<R, E> for Join<T>
where
    T: crate::keyed::Keyed + TreeMerge<R, E>,
    E: std::convert::From<ToqlError>,
{
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
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        match self {
            Join::Key(_) => Ok(()),
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
