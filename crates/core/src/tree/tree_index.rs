
use crate::query::field_path::Descendents;
use std::collections::HashMap;
use std::result::Result;
use crate::from_row::FromRow;

pub trait TreeIndex<R>
where Self: FromRow<R>
{

    fn index<'a>(&self, descendents: &Descendents<'a>,  rows: &[R], index: &mut HashMap<u64,Vec<usize>>) 
    -> Result<(), <Self as FromRow<R>>::Error>;
}

