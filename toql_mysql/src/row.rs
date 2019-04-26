
use mysql;

// Helper functions to load query result into structures

pub trait FromResultRow<T> {
    fn forward_row( i: usize)-> usize;
    fn from_row_with_index( row: &mut mysql::Row, i: &mut usize) -> Result<T,mysql::error::Error> ;
}

pub fn from_query_result<T: FromResultRow<T>>(result: mysql::QueryResult) -> Result<Vec<T>, mysql::error::Error> {
    let mut i: usize = 0;
    result
        .map(|row| { i = 0; T::from_row_with_index( &mut row?, &mut i)})
        .collect()
}

pub fn from_row<T: FromResultRow<T>>(mut row: mysql::Row) -> Result<T, mysql::error::Error> {
    let mut i: usize = 0;
    T::from_row_with_index(&mut row, &mut i)
}


