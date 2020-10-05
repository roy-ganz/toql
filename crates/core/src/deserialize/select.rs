use super::row::Row;
use crate::error::Result;
use crate::sql::Sql;

// Runs a query on a database and returns an iterator to the resulting rows.
// With to row trait the expected structs can be read
// IMplemented by database support.
trait Select<T, R, I>
where
    R: Row<T>,
    I: Iterator<Item = R>,
{
    fn select(sql: Sql) -> Result<I>;
}
