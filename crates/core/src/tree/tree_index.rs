use crate::from_row::FromRow;
use crate::query::field_path::Descendents;
use std::collections::HashMap;
use std::result::Result;

// R is database specific row
// Trait is implemented for structs that can deserialize from rows
pub trait TreeIndex<R>
where
    Self: FromRow<R>,
{
    fn index<'a, I>(
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: I,
        index: &mut HashMap<u64, Vec<usize>>,
    ) -> Result<(), <Self as FromRow<R>>::Error>
     where I: IntoIterator<Item=R>;
}
