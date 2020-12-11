use crate::from_row::FromRow;

use super::Join;
use crate::key::Keyed;

use crate::sql_builder::select_stream::Select;


 impl<T, R, E> FromRow <R, E> for Join<T> 
 where T:Keyed + FromRow<R, E>
 {
     
     fn from_row_with_index<'a, I>(
        row: &R,
        i: &mut usize,
        iter: &mut I,
    ) -> Result<Self,E>
    where
        I: Iterator<Item = &'a Select>,
        Self: Sized {
        
        Ok(Join::Entity(<T as FromRow<R, E>>::from_row_with_index(row, i, iter)?))
    }

 }