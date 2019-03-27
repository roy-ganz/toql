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
//! Toql consists of four parts:
//! 
//!  * A query language
//!  * A mapper that knows how to map query fields into table columns
//!  * A query builder that turn you query into sql using the mapper
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
extern crate pest;
#[macro_use]
extern crate pest_derive;



//pub mod toql;
pub mod buildable;
pub mod user_query;

pub mod sql_mapper;
pub mod sql_builder;
pub mod sql_builder_result;
pub mod query;
pub mod query_parser;
pub mod query_builder;

#[cfg(feature="mysqldb")]
pub mod mysql;

