
use mysql;

// Helper functions to load query result into structures

pub trait FromResultRow<T> {
    fn from_row_with_index(row: mysql::Row, i: &mut usize) -> Result<T,mysql::error::Error> ;
}

pub fn load<T: FromResultRow<T>>( result: Result<mysql::QueryResult, mysql::error::Error>) -> Result<Vec<T>,mysql::error::Error> {
    result.map(|r| from_query_result::<T>(r))?
}

pub fn from_query_result<T: FromResultRow<T>>(result: mysql::QueryResult) -> Result<Vec<T>, mysql::error::Error> {
    let mut i: usize = 0;
    result
        .map(|row| { i = 0; T::from_row_with_index(row?, &mut i)})
        .collect()
}

pub fn from_row<T: FromResultRow<T>>(row: mysql::Row) -> Result<T, mysql::error::Error> {
    let mut i: usize = 0;
    T::from_row_with_index(row, &mut i)
}
