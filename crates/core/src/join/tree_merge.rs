
use super::Join;
use crate::tree::tree_merge::TreeMerge;
use crate::error::ToqlError;
use crate::query::field_path::Descendents;
use crate::sql_builder::select_stream::SelectStream;
use std::collections::HashMap;


impl<T, R, E> TreeMerge<R, E> for Join<T>
where
    T: crate::keyed::Keyed + TreeMerge<R, E>,
    E: std::convert::From<ToqlError>
{
    fn merge<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &HashMap<u64, Vec<usize>>,
        selection_stream: &SelectStream,
    ) -> Result<(), E> {
        match self {
            Join::Key(_) => { Err(ToqlError::ValueMissing(  descendents.next().unwrap_or_default().to_string()).into())}
            Join::Entity(e) => {
                e.merge(descendents, field, rows, row_offset, index, selection_stream)
            }
        }
    }
}