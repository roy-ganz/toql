
use crate::query::field_path::Descendents;
use std::collections::HashMap;
use crate::error::Result;

pub trait TreeIndex
{
    fn index<'a>(&self,  descendents: &Descendents<'a>, index: &mut HashMap<u64,Vec<usize>>) -> Result<()>;
}

