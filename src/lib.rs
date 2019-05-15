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
//! There is a [guide](http://github.com/roy-ganz/toql/guide/index.html) that is better to get started.
//! 
//! ## Overview 
//! 
//! The project consists of the following main parts:
//!
//!  * A [Query Parser](../toql_core/query_parser/index.html) to build a Toql query from a string. 
//!  * A [Query](../toql_core/query/index.html) that can be built with methods.
//!  * An [SQL Mapper](../toql_core/sql_mapper/index.html) to map Toql fields to database columns or expressions.
//!  * An [SQL Builder](../toql_core/sql_builder/index.html) to  turn your Toql query into an SQL statement using the mapper.
//!  * A [Toql Derive](../toql_derive/index.html) to build all the boilerplate code to make some ✨ happen.
//!  * Integration with
//!      * [MySQL](../toql_mysql/index.html)
//!      * [Rocket](../toql_rocket/index.html)
//!
//! ## Small Example
//! Using Toql without any dependency features is possible and easy. Here we go:
//! ```
//! let query = QueryParser::parse("id, +title LK %foo%").unwrap();
//! let mapper = Mapper::new("Book b");
//!     mapper
//!         .map_field("id", "b.id")
//!         .map_field("title", "b.title");
//!
//! let result = SqlBuilder::new().build(&mapper, &query).unwrap();
//! assert_eq!("SELECT id, title FROM Book b WHERE b.title LIKE ? ORDER BY title ASC", result.to_sql());
//! ```
//! 
//! ## Bigger Example
//! However using the Rocket and MySQL integration will reduce your amount of coding to a minimum. 
//! If you have a MySQL server running, check out the full CRUD example with:
//! 
//! ```bash
//! ROCKET_DATABASES={example_db={url=mysql://USER:PASS@localhost:3306/example_db}} cargo +nightly run --example crud_rocket_mysql
//! 
//! ```
//! 

pub use toql_core::error;
pub use toql_core::error::Result;
pub use toql_core::query;
pub use toql_core::query_parser;
pub use toql_core::sql_builder;
pub use toql_core::sql_builder_result;
pub use toql_core::sql_mapper;
pub use toql_core::fields_type;
pub use toql_core::merge;
pub use toql_core::indelup;


pub use toql_derive as derive;


pub use log; // Reexport for generated code from Toql derive




#[cfg(feature = "rocket_mysql")]
pub use toql_rocket as rocket;

#[cfg(any(feature = "mysql", feature = "rocket_mysql"))]
pub use toql_mysql as mysql;
