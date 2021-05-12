//#![feature(const_generics)]

extern crate pest;

// Reexports
#[cfg(feature = "serde_feature")]
pub extern crate serde; // For generated keys and Join<T>

#[macro_use]
pub mod error;
pub mod alias;
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

#[macro_use]
pub mod log_helper;

extern crate lazy_static;

pub mod merge;
pub mod query;
pub mod query_fields;
pub mod query_parser;
pub mod query_path;
pub mod sql_builder;
//pub mod sql_builder_new;
pub mod sql_expr;
pub mod sql_expr_parser;
//pub mod sql_expr:resolver; // Bug?
pub mod alias_translator;
pub mod parameter_map;

//pub mod sql_builder_result;
pub mod field_handler;
pub mod from_row;
pub mod join_handler;
pub mod predicate_handler;
pub mod sql_mapper;
pub mod sql_mapper_registry;

//pub mod path_predicate;
pub mod tree;

pub mod backend;
pub mod join;
//pub mod update_field;
//pub mod insert_path;
pub mod cache;
pub mod cache_builder;
pub mod role_expr;
pub mod role_expr_parser;
pub mod role_validator;

pub use log; // Reexport for generated code from Toql derive
