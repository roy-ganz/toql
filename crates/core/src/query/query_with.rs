
/// A trait to convert any structure into a Query. 
/// For emxaple implement this for your configuration
/// and you can do `Query::new().with(config)`

use super::Query;

pub trait QueryWith<T> {
    fn with(&self, query: Query<T>) -> Query<T>;
}