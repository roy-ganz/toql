use crate::from_row::FromRow;

use super::Join;
use crate::keyed::Keyed;

use crate::{error::ToqlError, sql_builder::select_stream::Select};


 impl<T, R, E> FromRow <R, E> for Join<T> 
 where T:Keyed + FromRow<R, E>, E: std::convert::From<ToqlError>
 {
     fn  forward<'a, I>(iter: &mut I) -> usize 
       where
        I: Iterator<Item = &'a Select>,Self: std::marker::Sized
     {
         
         <T as FromRow<R, E>>::forward(iter)
     }
     fn from_row<'a, I>(
        row: &R,
        i: &mut usize,
        iter: &mut I,
    ) -> Result<Option<Self>,E>
    where
        I: Iterator<Item = &'a Select>,
        Self: Sized {
        
        Ok(<T as FromRow<R, E>>::from_row(row, i, iter)?.map(|e| Join::Entity(Box::new(e))))
    }
    
 }