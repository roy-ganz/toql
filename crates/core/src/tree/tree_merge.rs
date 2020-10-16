use crate::{
    from_row::FromRow, query::field_path::Descendents, sql_builder::select_stream::SelectStream,
};
use std::{collections::HashMap, ops::Index};

/* pub trait RowIndex<R> {

    fn get_row(&self, pos:usize) -> Option<&R>;

}

impl<R> RowIndex<R> for Vec<R> {


    fn get_row(&self, pos:usize) -> Option<&R> {
       self.get(pos)
    }

}
impl<R> RowIndex<R> for &Vec<R> {


    fn get_row(&self, pos:usize) -> Option<&R> {
       self.get(pos)
    }

}  */

// R is database specific row
// Trait is implemented for structs that can deserialize from rows
pub trait TreeMerge<R, E> {
    fn merge<'a>(
        &mut self,
        descendents: &mut Descendents<'a>,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &HashMap<u64, Vec<usize>>,
        selection_stream: &SelectStream,
    ) -> Result<(), E>;
}
