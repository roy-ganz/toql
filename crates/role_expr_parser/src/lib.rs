//!
//! The query parser can turn a string that follows the Toql query syntax into a [Query](../query/struct.Query.html).
//!
//! ## Example
//!
//! ``` ignore
//! let  query = QueryParser::parse("*, +username").unwrap();
//! assert_eq!("*, +username", query.to_string());
//! ```
//! Read the guide for more information on the query syntax.
//!
//! The parser is written with [Pest](https://pest.rs/) and is fast. It should be used to parse query request from users.
//! To build a query within your program, build it programmatically with the provided methods.
//! This avoids typing mistakes and - unlike parsing - cannot fail.
//!

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "role_expr.pest"]
pub struct PestRoleExprParser;
