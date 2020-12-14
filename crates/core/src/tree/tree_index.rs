use crate::query::field_path::Descendents;
use std::collections::HashMap;
use std::result::Result;
use crate::{error::ToqlError};

// R is database specific row, E the desired output error
// Trait is implemented for structs that can deserialize from rows
pub trait TreeIndex<R, E> 
 where  E: std::convert::From<ToqlError>
{
    fn index<'a>(
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>;
}
