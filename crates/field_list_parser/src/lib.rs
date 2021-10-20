//! The field list parser is used by the `fields!` macro to compile a list of Toql fields into programm code.
//!
//! The parser is written with [Pest](https://pest.rs/).


extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "field_list.pest"]
pub struct PestFieldListParser;
