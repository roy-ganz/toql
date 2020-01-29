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
pub fn from_query_result<T: FromResultRow<T>>(result: mysql::QueryResult) -> Result<Vec<T>> {
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

/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result_with_primary_keys<
    T: FromResultRow<T> + toql_core::key::Key,
    J: FromResultRow<J>,
>(
    result: mysql::QueryResult,
) -> Result<(Vec<T>, Vec<J>)> {
    let mut entities: Vec<T> = Vec::new();
    let mut pkeys: Vec<J> = Vec::new();

    for row in result {
        let mut i: usize = 0;
        let mut r = row?;
        entities.push(T::from_row_with_index(&mut r, &mut i)?);
        i = i + 1;
        pkeys.push(J::from_row_with_index(&mut r, &mut i)?);
    }

    Ok((entities, pkeys))
}
/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result_with_merge_keys<
    T: FromResultRow<T> + toql_core::key::Key,
    J: FromResultRow<J>,
    K: FromResultRow<K>,
>(
    result: mysql::QueryResult,
) -> Result<(Vec<T>, Vec<J>, Vec<K>)> {
    let mut entities: Vec<T> = Vec::new();
    let mut pkeys: Vec<J> = Vec::new();
    let mut keys: Vec<K> = Vec::new();

    for row in result {
        let mut i: usize = 0;
        let mut r = row?;
        entities.push(T::from_row_with_index(&mut r, &mut i)?);
        i = i + 1;
        pkeys.push(J::from_row_with_index(&mut r, &mut i)?);
        i = i + 1;
        keys.push(K::from_row_with_index(&mut r, &mut i)?);
    }
    /* result
    .map(|row| {
        i = 0;
        let mut r = row?;
        Ok((T::from_row_with_index(&mut r, &mut i)?,
        K::from_row_with_index(&mut r, &mut i)?
       ))
    })
    .collect() */
    Ok((entities, pkeys, keys))
}

/*
/// Function to convert MySQL query result into Toql struct.
pub fn from_query_result_with_keys<T: FromResultRow<T>, F: Fn (&mut mysql::Row, &mut usize)->(J,K), J, K >(
    result: mysql::QueryResult, keys: F
) -> Result<Vec<(T,(J,K))>>

{
    let mut i: usize = 0;
    result
        .map(|row| {
            i = 0;
            let mut r = row?;
            (T::from_row_with_index(&mut r, &mut i)
                .and_then(|e| {
                    Ok((e, keys(&mut r, &mut i)))
                })  )
        })
        .collect()
} */
