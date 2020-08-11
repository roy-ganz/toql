use mysql;

use crate::error::Result;

/// Trait to convert MySQL result row into Toql structs.
/// This is implements by Toql Derive for all dervied structs.
pub trait FromResultRow<T> {
    // Skip row values for struct.
    // Returns a new index that points to next struct.
   // fn forward_row(i: usize) -> usize;
    // Read row values into struct, starting from index `i`.
    fn from_row_with_index<'a, I>(row: &mut mysql::Row, i: &mut usize, iter: &mut I ) -> Result<T>
    where I: Iterator<Item = &'a bool>;
}

/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result<'a, T,I>(result: mysql::QueryResult, mut iter: &mut I) -> Result<Vec<T>>
where T: FromResultRow<T>, I: Iterator<Item = &'a bool> {
    let mut i: usize = 0;
    result
        .map(|row| {
            i = 0;
            T::from_row_with_index(&mut row?, &mut i, &mut iter)
        })
        .collect()
}

/// Function to convert MySQL query result row into Rust struct.
pub fn from_row<'a, T, I>(mut row: mysql::Row,  mut iter: &mut I) -> Result<T>
where T: FromResultRow<T>, I: Iterator<Item = &'a bool> {
    let mut i: usize = 0;
    T::from_row_with_index(&mut row, &mut i, &mut iter)
}

/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result_with_primary_keys<'a, T,J,I>( result: mysql::QueryResult, mut iter :  &mut I) -> Result<(Vec<T>, Vec<J>)> 
where T: FromResultRow<T>,   J: FromResultRow<J>,  I: Iterator<Item = &'a bool>
{
    let mut entities: Vec<T> = Vec::new();
    let mut pkeys: Vec<J> = Vec::new();

    for row in result {
        let mut i: usize = 0;
        let mut r = row?;
        entities.push(T::from_row_with_index(&mut r, &mut i, &mut iter)?);
        
        pkeys.push(J::from_row_with_index(&mut r, &mut i, &mut iter)?);
    }

    Ok((entities, pkeys))
}
/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result_with_merge_keys<'a, T,J,K,I>(result: mysql::QueryResult, mut iter:  &mut I) -> Result<(Vec<T>, Vec<J>, Vec<K>)> 
where  T: FromResultRow<T>, J: FromResultRow<J>, K: FromResultRow<K>, I: Iterator<Item = &'a bool>
{
    let mut entities: Vec<T> = Vec::new();
    let mut pkeys: Vec<J> = Vec::new();
    let mut keys: Vec<K> = Vec::new();

    for row in result {
        let mut i: usize = 0;
        let mut r = row?;
        entities.push(T::from_row_with_index(&mut r, &mut i, &mut iter)?);
     
        pkeys.push(J::from_row_with_index(&mut r, &mut i, &mut iter)?);
     
        keys.push(K::from_row_with_index(&mut r, &mut i, &mut iter)?);
    }
    
    Ok((entities, pkeys, keys))
}
