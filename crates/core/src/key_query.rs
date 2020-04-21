
use crate::query::Query;

/// Trait for keys to be converted to foreign queries. 
pub trait KeyQuery {
    fn foreign_query<T>(path:&str) -> Query<T>;
}

