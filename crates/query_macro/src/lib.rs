//! The `query!` macro compiles a Toql query into program code. 
//! Any syntax errors, wrong paths or field names will show up at compile time.
//! Wrong paths or field names are detected because the query! macro uses the 
//! query builder functions that are genereated by the Toql derive.
//!
//! ### Example
//! ```rust
//! let f = query!(User, "id eq ?, address_title", 42);
//! ```


#![recursion_limit = "512"]

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse_macro_input;
use proc_macro::TokenStream;

mod query_macro;

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init

    let ast = parse_macro_input!(input as query_macro::QueryMacro);
    let gen = query_macro::parse(&ast.query, ast.struct_type, &mut ast.arguments.iter());
  
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
