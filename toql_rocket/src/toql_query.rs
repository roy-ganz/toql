use toql_core::query::Query;
use rocket::FromForm;

/// Struct to hold URL query parameters
/// 
/// This allows to query entities with optional query parameters.
/// Ensure that you URL encode your query! 
/// Instead of `query=*` you must write `query=%2A`.
/// 
/// ### Examples of URL queries for Toql
/// ```url
///  ..?query=id%2C%20%2Busername&first=5&max=20&count=false
///  ..?max=5
///  ..?query=%2A,phone_%2A,count=false
/// ```
#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    /// The actual Toql query, like `id, +username, phone_*`
    /// 
    /// Default `Some("*")`
    pub query: Option<Query>,
    /// The offset to the first record. For example 10 will skip the first 10 records.
    /// 
    /// Default `Some(0)`
    pub first: Option<u64>,
    /// The maximum number of records to return.
    /// 
    /// Default `Some(10)`
    pub max: Option<u16>,
    /// Get filtered count and total count.
    /// 
    /// Default `Some(true)`
    pub count: Option<bool>,
}

