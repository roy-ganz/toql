use super::Join;
use crate::error::ToqlError;
use crate::keyed::Keyed;
use crate::query::field_path::FieldPath;
use crate::tree::tree_index::TreeIndex;
use std::collections::HashMap;

impl<T, R, E> TreeIndex<R, E> for Join<T>
where
    T: Keyed + TreeIndex<R, E>,
    E: std::convert::From<ToqlError>,
{
    fn index<'a, I>(
        descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>
    where
        I: Iterator<Item = FieldPath<'a>> + Clone,
    {
        T::index(descendents, rows, row_offset, index)
    }
}
