//#![feature(const_generics)]

extern crate pest;

// Reexports
#[cfg(feature = "serde_feature")]
pub extern crate serde; // For generated keys and Join<T>

#[macro_use]
pub mod error;
pub mod alias_format;
pub mod deserialize;
pub mod identity;
pub mod key;
pub mod key_fields;
pub mod keyed;
pub mod map_key;
//pub mod map_query;
pub mod from_iterator;
pub mod page;
pub mod page_counts;
pub mod result;
pub mod sql;
pub mod sql_arg;
pub mod toql_api;

#[macro_use]
pub mod log_macro;
#[macro_use]
pub mod join_macro;
#[macro_use]
pub mod val_macro;
#[macro_use]
pub mod none_error_macro;

extern crate lazy_static;

pub mod alias_translator;
pub mod backend;
pub mod cache;
pub mod cache_builder;
pub mod field_handler;
pub mod from_row;
pub mod join;
pub mod join_handler;
pub mod parameter_map;
pub mod predicate_handler;
pub mod query;
pub mod query_fields;
pub mod query_parser;
pub mod query_path;
pub mod role_expr;
pub mod role_expr_parser;
pub mod role_validator;
pub mod sql_builder;
pub mod sql_expr;
pub mod table_mapper;
pub mod table_mapper_registry;
pub mod tree;

pub use tracing; // Reexport dependency for generated code from Toql derive
