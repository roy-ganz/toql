//! Rocket integration of Toql.
//! This contains 
//!  - A high level function to query Toql structs.
//!  - Query parameters.
//!  - Support to add counting information to HTTP response headers
//!
//! This allows to query Toql structs like this
//! 
//! ```ignore
//! #[macro_use]
//! extern crate rocket;
//! #[macro_use]
//! extern crate rocket_contrib;
//! 
//! use toql::sql_mapper::SqlMapperCache;
//! use toql::rocket::{ToqlQuery, Counted};
//! use rocket::request::Form;
//! use myql::Conn;
//! use rocket_contrib::json::Json;
//! use toql::rocket::mysql::load_many;
//! 
//! #[database("example_db")]
//! struct ExampleDbConnection(mysql::Conn);
//! 
//! struct User {id:u64, username: Option<String>};
//! 
//! #[get("/?<toql..>")]
//! fn query( mappers: State<SqlMapperCache>,
//!               conn: ExampleDbConnection, 
//!               toql: Form<ToqlQuery>)
//! -> Result<Counted<Json<Vec<User>>>> {
//!    let ExampleDbConnection(mut c) = conn;
//!
//!    let r = load_many::<User>(&toql, &mappers, &mut c)?;
//!    Ok(Counted(Json(r.0), r.1))
//! }
//! 
//! ```
//! 
//! 

pub mod counted;
pub mod toql_query;


#[cfg(feature = "mysqldb")]
pub mod mysql;

pub use counted::Counted;
pub use toql_query::ToqlQuery;