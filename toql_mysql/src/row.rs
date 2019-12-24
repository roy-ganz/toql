
use mysql;

use crate::error::Result;

/// Trait to convert MySQL result row into Toql structs.
/// This is implements by Toql Derive for all dervied structs.
pub trait FromResultRow<T> {
    // Skip row values for struct.
    // Returns a new index that points to next struct.
    fn forward_row(i: usize) -> usize;
    // Read row values into struct, starting from index `i`.
    fn from_row_with_index(row: &mut mysql::Row, i: &mut usize) -> Result<T>;
}

/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result<T: FromResultRow<T>>(
    result: mysql::QueryResult,
) -> Result<Vec<T>> {
    let mut i: usize = 0;
    result
        .map(|row| {
            i = 0;
            T::from_row_with_index(&mut row?, &mut i)
        })
        .collect()
}

/// Function to convert MySQL query result row into Rust struct.
pub fn from_row<T: FromResultRow<T>>(mut row: mysql::Row) -> Result<T> {
    let mut i: usize = 0;
    T::from_row_with_index(&mut row, &mut i)
}
