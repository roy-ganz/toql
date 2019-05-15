//! Rocket integration of Toql.
//! This contains 
//!  - A high level function to query Toql structs.
//!  - Query parameters.
//!  - Support to add counting information to HTTP response headers
//!
//! This allows to query Toql structs like this
//! 
//! ```rust
//! #[get("/?<toql..>")]
//! pub fn query( mappers: State<SqlMapperCache>,
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