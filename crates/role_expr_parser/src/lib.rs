//! The role expression parser is used by the `role_expr!` macro to compile a role expression into programm code.
//!
//! The parser is written with [Pest](https://pest.rs/).

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "role_expr.pest"]
pub struct PestRoleExprParser;
