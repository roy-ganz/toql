use super::Join;
use crate::keyed::Keyed;
use crate::tree::tree_index::TreeIndex;
use crate::query::field_path::{FieldPath, Descendents};
use std::collections::HashMap;
use crate::{error::ToqlError};

impl<T, R, E> TreeIndex<R, E> for Join<T>
where
    T: Keyed + TreeIndex<R,E>,
    E: std::convert::From<ToqlError>
{

    fn index<'a, I>(
        descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>  where I: Iterator<Item = FieldPath<'a>>{
        T::index(descendents, rows, row_offset, index)
    }
}