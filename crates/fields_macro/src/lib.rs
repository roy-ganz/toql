//! The `fields!` macro compiles a list of Toql field names into program code. 
//! Any syntax errors or wrong field names will show up at compile time.
//!
//! Wrong field names are detected because the `field!` macro uses the 
//! query builder functions that are genereated by the Toql derive.
//!
//! ### Example
//! Assume a `struct User` with a joined `address`.
//! ```rust
//! let f = fields!(User, "*, address_title");
//! ```
//!
//! Notice that the `fields!` macro takes a type, however the resulting `Fields` is untyped. 
//! This is a shortcoming and will be resolved in the future.

#![recursion_limit = "512"]

extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse_macro_input;

use proc_macro::TokenStream;

mod fields_macro;

#[proc_macro]
pub fn fields(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init

    tracing::debug!("Source code for `{}`:\n", &input);
    let ast = parse_macro_input!(input as fields_macro::FieldsMacro);

    let gen = match ast {
        fields_macro::FieldsMacro::FieldList { struct_type, query } => {
            fields_macro::parse(&query, struct_type)
        }
        fields_macro::FieldsMacro::Top => Ok(quote!(toql::toql_api::fields::Fields::from(vec![
            "*".to_string()
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
