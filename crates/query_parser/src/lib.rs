//! The query parser is used by the `query!` macro to compile a Toql query into programm code.
//!
//! The parser is written with [Pest](https://pest.rs/).

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "toql.pest"]
pub struct PestQueryParser;
