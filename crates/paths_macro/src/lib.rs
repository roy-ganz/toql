//!
//! The Toql Derive creates all the boilerplate functions to make the ✨ happen.
//! Using the derive is the easiest way to deal with your structs and is therefore recommended.
//! However beware that the generated code size can become large as it's about ~9K lines of code for a small struct.
//! You may disable some functionality.
//!
//! For a derived struct the following is generated:
//!  - Trait [Mapped](../toql_core/sql_mapper/trait.Mapped.html) to map struct to [SqlMapper](../toql_core/sql_mapper/struct.SqlMapper.html).
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
#![feature(proc_macro_span)]

extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse_macro_input;

use proc_macro::TokenStream;

mod paths_macro;

#[proc_macro]
pub fn paths(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
                                    // eprintln!("{:?}", input);

    let ast = parse_macro_input!(input as paths_macro::PathsMacro);

    let gen = match ast {
        paths_macro::PathsMacro::PathList { struct_type, query } => {
            paths_macro::parse(&query, struct_type)
        }
        paths_macro::PathsMacro::Top => Ok(quote!(toql::backend::paths::Paths::from(vec![
            "".to_string()
        ]))),
    };

    let source = proc_macro::Span::call_site()
        .source_text()
        .unwrap_or("".to_string());

    match gen {
        Ok(o) => {
            log::debug!("Source code for `{}`:\n{}", source, o.to_string());
            TokenStream::from(o)
        }
        Err(e) => {
            log::debug!("Source code for `{}`:\n{}", source, e.to_string());
            TokenStream::from(e)
        }
    }
}
