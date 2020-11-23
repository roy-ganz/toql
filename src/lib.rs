// Toql. Transfer Object Query Language
// Copyright (c) 2019 Roy Ganz
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! # Toql. Transfer Object Query Language
//!
//! Welcome to Toql API documentation!
//!
//! This API documentation is very technical and is purely a reference.
//! There is a [guide](https://roy-ganz.github.io/toql/) that is better to get started.
//!
//! ## Overview
//!
//! The project consists of the following main parts:
//!
//!  * A [Query Parser](https://docs.rs/toql_core/0.1/toql_core/query_parser/index.html) to build a Toql query from a string.
//!  * A [Query](https://docs.rs/toql_core/0.1/toql_core/query/index.html) that can be built with methods.
//!  * A [SQL Mapper](https://docs.rs/toql_core/0.1/toql_core/sql_mapper/index.html) to map Toql fields to database columns or expressions.
//!  * A [SQL Builder](https://docs.rs/toql_core/0.1/toql_core/sql_builder/index.html) to  turn your Toql query into an SQL statement using the mapper.
//!  * A [Toql Derive](https://docs.rs/toql_derive/0.1/index.html) to build all the boilerplate code to make some ✨ happen.
//!  * Integration with
//!      * [MySQL](https://docs.rs/toql_mysql/0.1/index.html)
//!      * [Rocket](https://docs.rs/toql_rocket/0.1/index.html)
//!
//! ## Small Example
//! Using Toql without any dependency features is possible and easy. Here we go:
//! ``` rust
//! use toql::{query_parser::QueryParser, sql_mapper::SqlMapper, sql_builder::SqlBuilder};
//!
//! let query = QueryParser::parse("id, +title LK '%foo%'").unwrap();
//! let mut mapper = SqlMapper::new("Book b");
//!     mapper
//!         .map_field("id", "b.id")
//!         .map_field("title", "b.title");
//!
//! let result = SqlBuilder::new().build(&mapper, &query).unwrap();
//! assert_eq!("SELECT b.id, b.title FROM Book b WHERE b.title LIKE ? ORDER BY b.title ASC", result.to_sql());
//! ```
//!
//! ## Bigger Example
//! Have a look at the [CRUD example](https://github.com/roy-ganz/toql/blob/master/examples/rocket_mysql/main.rs) that serves users with Rocket and MySQL.
//!

pub use toql_core::alias;
pub use toql_core::alias_translator;
pub use toql_core::error;
pub use toql_core::error::Result;
pub use toql_core::from_row;
pub use toql_core::key;
pub use toql_core::merge;
pub use toql_core::query;
pub use toql_core::sql;
pub use toql_core::sql_arg;
pub use toql_core::sql_expr;
pub use toql_core::sql_expr_parser;
pub use toql_core::paths;
pub use toql_core::fields;

pub use toql_core::log_sql; // Export macro
pub use toql_core::ok_or_fail; // Export macro

pub use toql_core::field_handler;
pub use toql_core::join_handler;
pub use toql_core::predicate_handler;
pub use toql_core::query_fields;
pub use toql_core::query_path;
pub use toql_core::query_parser;
pub use toql_core::sql_builder;
pub use toql_core::sql_mapper;
pub use toql_core::sql_mapper_registry;
pub use toql_core::to_query;
pub use toql_core::tree;
pub use toql_core::backend;
pub use toql_core::update_field;
pub use toql_core::insert_path;
pub use toql_core::join;

pub use toql_derive as derive;
pub use toql_query_macro as query_macro;
pub use toql_fields_macro as fields_macro;
pub use toql_paths_macro as paths_macro;
pub use toql_role_expr_macro as role_expr_macro;

pub use toql_core::role_expr;
pub use toql_core::role_expr_parser;
pub use toql_core::role_validator;
pub use toql_core::cache;
pub use toql_core::page;


pub use toql_core::log; // Reexport for derive



#[cfg(feature = "mysql15")]
pub use toql_mysql as mysql;


pub mod prelude;
