//!
//! The Toql Derive creates all the boilerplate functions to make the ✨ happen.
//! Using the derive is the easiest way to deal with your structs and is therefore recommended.
//! However beware that the generated code size can become large as it's about ~9K lines of code for a small struct.
//! You may disable some functionality.
//!
//! For a derived struct the following is generated:
//!  - Trait [Mapped](../toql_core/table_mapper/trait.Mapped.html) to map struct to [TableMapper](../toql_core/table_mapper/struct.TableMapper.html).
//!  - Methods for all fields to support building a [Query](../toql_core/query/struct.Query.html).
//!  - Methods to load, insert, delete and update a struct. Requires database feature.
//!
//! ### Example:
//! ```rust
//! use toql::derive::Toql;
//!
//! #[derive(Toql)]
//! struct User {

//!   #[toql(key)] // Use this field as key for delete and update
//!   id : u64,
//!
//!   username : Option<String>
//! }
//! ```
//!
//! Check out the [guide](https://roy-ganz.github.io/toql/derive/reference.html) for list of available attributes.
//!

#![recursion_limit = "512"]

extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse_macro_input;

use proc_macro::TokenStream;

mod sql_expr_macro;

#[proc_macro]
pub fn sql_expr(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
                                    // eprintln!("{:?}", input);

    let ast = parse_macro_input!(input as sql_expr_macro::SqlExprMacro);

    let gen = sql_expr_macro::parse(&ast.query, &mut ast.arguments.iter());

    match gen {
        Ok(o) => {
            tracing::debug!(
                "Source code for `{}`:\n{}",
                ast.query.value(),
                o.to_string()
            );
            TokenStream::from(o)
        }
        Err(e) => {
            tracing::debug!(
                "Source code for `{}`:\n{}",
                ast.query.value(),
                e.to_string()
            );
            TokenStream::from(e)
        }
    }
}
