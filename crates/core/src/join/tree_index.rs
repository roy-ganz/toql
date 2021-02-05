use super::Join;
use crate::keyed::Keyed;
use crate::tree::tree_index::TreeIndex;
use crate::query::field_path::Descendents;
use std::collections::HashMap;
use crate::{error::ToqlError};

impl<T, R, E> TreeIndex<R, E> for Join<T>
where
    T: Keyed + TreeIndex<R,E>,
    E: std::convert::From<ToqlError>
{

    fn index<'a>(
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>  {
        T::index(descendents, field, rows, row_offset, index)
    }
}