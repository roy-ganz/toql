//! The `paths!` macro compiles a list of Toql field names into program code. 
//! Any syntax errors or wrong path names will show up at compile time.
//!
//! Wrong path names are detected because the `paths!` macro uses the 
//! query builder functions that are genereated by the Toql derive.
//!
//! ### Example
//! Assume a `struct User` with a joined `address`.ok_or(none_error!())?
//! ```rust
//! let f = paths!(User, "address");
//! ```
//!
//! Notice that the `paths!` macro takes a type, however the resulting `Paths` is untyped. 
//! This is a shortcoming and will be resolved in the future.

#![recursion_limit = "512"]

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
                                    
    tracing::debug!("Source code for `{:?}`: ", &input);

    let ast = parse_macro_input!(input as paths_macro::PathsMacro);

    let gen = match ast {
        paths_macro::PathsMacro::PathList { struct_type, query } => {
            paths_macro::parse(&query, struct_type)
        }
        paths_macro::PathsMacro::Top => Ok(quote!(toql::toql_api::paths::Paths::from(vec![
            "".to_string()
        ]))),
    };

    match gen {
        Ok(o) => {
            tracing::debug!("{}", o.to_string());
            TokenStream::from(o)
        }
        Err(e) => {
            tracing::debug!("{}", e.to_string());
            TokenStream::from(e)
        }
    }
}
