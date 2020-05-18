//#![feature(const_generics)]

extern crate pest;


#[macro_use]
pub mod error;
pub mod alias;
pub mod dialect;
pub mod key;
pub mod load;
pub mod mutate;
pub mod select;
pub mod sql;
pub mod to_query;

#[macro_use]
pub mod log_helper;

#[macro_use]
extern crate lazy_static;

pub mod merge;
pub mod query;
pub mod query_fields;
pub mod query_parser;
pub mod sql_builder;
//pub mod sql_builder_new;
pub mod sql_expr;
pub mod alias_translator;
pub mod parameter;

//pub mod sql_builder_result;
pub mod sql_mapper;
pub mod sql_mapper_registry;
pub mod field_handler;
pub mod join_handler;
pub mod predicate_handler;


pub use log; // Reexport for generated code from Toql derive
