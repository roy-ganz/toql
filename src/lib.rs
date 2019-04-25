// toql. Transfer Object Query Language
// Copyright (c) 2019 Roy Ganz
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! # toql. Transfer Object Query Language
//!
//! toql is a way to build SQL-queries. It can be useful for web clients making queries to a web server.
//! Toql consists of five parts:
//!
//!  * A query language that can be parsed
//!  * A 
//!  * A mapper that knows how to translate query fields into table columns
//!  * A query builder that turn your query into SQL using the mapper
//!  * A Toql derive, to map big structs to tables and resolve dependencies
//!
//! ## Example
//! ```
//! let query = QueryParser::parse("id, +title LK '%foo%'").unwrap();
//! let mapper = Mapper::new();
//!     mapper
//!         .map_field("id", "id")
//!         .map_field("title", "title");
//!
//! let result = SqlBuilder::new().build(&mapper, &query).unwrap();
//! assert_eq!("SELECT id, title FROM Book WHERE title LIKE ? ORDER BY title ASC", result.sql_for_table("Book"));
//! ```
//!
//! ## Features
//!
//! * The query language allows selecting, filtering and ordering individual fields
//! * Subfields are supported
//! * Filter predicates can be joined by AND and OR. Parens are supported.
//! * Mapper allows mapping of single fields, structs (derive Toql) and custom operations
//! * Mapper can be cached for reuse, to make toql perform fast
//! * SQL builder can build normal queries, count queries and subpaths queries
//!
//! Important: Toql is not an ORM. Toql only cares about database queries, it cannot insert or update database tables.
//! If you are looking for an ORM, checkout Diesel.
//!
pub use toql_core::load;
pub use toql_core::query;
pub use toql_core::query_parser;
pub use toql_core::sql_builder;
pub use toql_core::sql_builder_result;
pub use toql_core::sql_mapper;
pub use toql_core::query_builder;

pub use toql_derive as derive;

pub use log; // Reexport for generated code from Toql derive

#[cfg(feature = "rocket_mysql")]
pub use toql_rocket as rocket;

#[cfg(any(feature = "mysql", feature = "rocket_mysql"))]
pub use toql_mysql as mysql;
