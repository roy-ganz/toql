//! The SQL expression parser is used by the `sql_expr!` macro to compile a SQL expression into programm code.
//!
//! The parser is written with [Pest](https://pest.rs/).

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "sql_expr.pest"]
pub struct PestSqlExprParser;
