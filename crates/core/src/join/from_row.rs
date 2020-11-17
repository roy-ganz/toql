use crate::from_row::FromRow;

use super::Join;
use crate::key::Keyed;

use crate::sql_builder::select_stream::Select;


 impl<T, R> FromRow <R> for Join<T> 
 where T:Keyed + FromRow<R>
 {
     type Error = <T as FromRow<R>>::Error;
     fn from_row_with_index<'a, I>(
        row: &R,
        i: &mut usize,
        iter: &mut I,
    ) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = &'a Select>,
        Self: Sized {
        
        Ok(Join::Entity(<T as FromRow<R>>::from_row_with_index(row, i, iter)?))
    }

 }