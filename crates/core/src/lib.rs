//#![feature(const_generics)]

extern crate pest;

// Reexports
#[cfg(feature = "serde_feature")]
pub extern crate serde; // For generated keys and Join<T>

#[macro_use]
pub mod error;
pub mod alias_format;
pub mod deserialize;
pub mod key;
pub mod key_fields;
pub mod keyed;
pub mod map_key;
pub mod page;
pub mod result;
//pub mod mutate;
//pub mod select;
pub mod map_query;
pub mod sql;
pub mod sql_arg;
pub mod to_query;
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

pub mod merge;
pub mod query;
pub mod query_fields;
pub mod query_parser;
pub mod query_path;
pub mod sql_builder;
pub mod sql_expr;
pub mod sql_expr_parser;
pub mod alias_translator;
pub mod parameter_map;

pub mod field_handler;
pub mod from_row;
pub mod join_handler;
pub mod predicate_handler;
pub mod table_mapper;
pub mod table_mapper_registry;

pub mod tree;

pub mod backend;
pub mod join;
pub mod try_join;
pub mod cache;
pub mod cache_builder;
pub mod role_expr;
pub mod role_expr_parser;
pub mod role_validator;

pub use log; // Reexport dependency for generated code from Toql derive
