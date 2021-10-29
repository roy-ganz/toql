//! # Toql - A friendly and productive ORM
//!
//!
//! [Beginner Guide](https://roy-ganz.github.io/toql), [API documentation](https://docs.rs/toql/0.3/toql/)
//!
//! Toql is an ORM for async databases that features
//! - Translation between Rust structs and database tables.
//! - Can load and modify nested structs.
//! - A unique dead simple query language, suitable for web clients.
//! - Different table aliases from long and readable to tiny and fast.
//! - Prepared statements against SQL injection.
//! - Support for raw SQL for full database power.
//! - Support for role based access.
//! - Highly customizable through user defined parameters, query functions, field handlers, etc.
//! - Compile time safety for queries, fields and path names.
//! - No unsafe Rust code.
//! - Tested on real world scenario.
//!
//! It currently only supports **MySQL**. More are coming, promised :)
//!
//! ## Installation
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! toql = {version = "0.3", features = ["serde"]}
//! toql_mysql_async = "0.3"
//! ```
//!
//! ## Look And Feel
//!
//! Derive your structs:
//! ```rust, ignore
//! #[derive(Toql)]
//! #[toql(auto_key = true)]
//! struct Todo {
//!     #[toql(key)]
//!     id: u64,
//!     what: String,
//!
//!     #[toql(join())]
//!     user: User
//! }
//! ```
//!
//! And do stuff with them:
//! ```rust, ignore
//! let toql = ...
//! let todo = Todo{ ... };
//!
//! // Insert todo and update its generated id
//! toql.insert_one(&mut todo, paths!(top)).await?;
//!
//! // Compile time checked queries!
//! let q = query!(Todo, "*, user_id eq ?", &todo.user.id);
//!
//! // Typesafe loading
//! let user = toql.load_many(q).await?;
//! ```
//!
//!
//! ## Quick start
//! Toql has a [supporting crate](https://crates.io/crates/toql_rocket) to play well with [Rocket](https://crates.io/crates/rocket). Check out the [CRUD example](https://github.com/roy-ganz/todo_rotomy).
//!
//! ## Contribution
//! Comments, bug fixes and quality improvements are welcome.
//!
//! ## License
//! Toql is distributed under the terms of both the MIT license and the
//! Apache License (Version 2.0).

pub use toql_core::alias_format;
pub use toql_core::alias_translator;
pub use toql_core::error;
pub use toql_core::from_row;
pub use toql_core::identity;
pub use toql_core::key;
pub use toql_core::key_fields;
pub use toql_core::keyed;
pub use toql_core::map_key;
pub use toql_core::query;
pub use toql_core::result;
pub use toql_core::sql;
pub use toql_core::sql_arg;
pub use toql_core::sql_expr;
pub use toql_core::toql_api::fields;
pub use toql_core::toql_api::paths;

pub use toql_core::log_sql; // Export macro for derives
pub use toql_core::none_error;
pub use toql_core::ok_or_fail; // Export macro (TODO: check for removal) // Export macro for macros

pub use toql_core::backend;
pub use toql_core::field_handler;
pub use toql_core::join_handler;
pub use toql_core::predicate_handler;
pub use toql_core::query_fields;
pub use toql_core::query_parser;
pub use toql_core::query_path;
pub use toql_core::sql_builder;
pub use toql_core::table_mapper;
pub use toql_core::table_mapper_registry;
pub use toql_core::tree;
//pub use toql_core::update_field;
//pub use toql_core::insert_path;
pub use toql_core::join;
//pub use toql_core::try_join;

pub use toql_derive as derive;
pub use toql_fields_macro as fields_macro;
pub use toql_paths_macro as paths_macro;
pub use toql_query_macro as query_macro;
pub use toql_role_expr_macro as role_expr_macro;
pub use toql_sql_expr_macro as sql_expr_macro;

pub use toql_core::cache;
pub use toql_core::deserialize;
pub use toql_core::page;
pub use toql_core::page_counts;
pub use toql_core::parameter_map;
pub use toql_core::role_expr;
//pub use toql_core::role_expr_parser;
pub use toql_core::role_validator;
pub use toql_core::mock_db;


pub use toql_core::toql_api; // Export for derives
pub use toql_core::tracing; // Reexport for derive

#[cfg(feature = "serde")]
pub use toql_core::serde; // Reexport for derive

pub use toql_core::log_literal_sql;
pub use toql_core::row; // For unit tests

pub mod prelude;
