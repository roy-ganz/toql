use crate::from_row::FromRow;
use crate::query::field_path::Descendents;
use std::collections::HashMap;
use std::result::Result;

// R is database specific row
// Trait is implemented for structs that can deserialize from rows
pub trait TreeIndex<R, E>
{
 
    
    fn index<'a>(
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: &[R], row_offset: usize,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), E>;
}
