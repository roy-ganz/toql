//! The `sql_expr!` macro compiles an SQL expression into program code. 
//!
//! ### Example
//! ```rust
//! let e = sql_expr!("SELECT ..id FROM User .. WHERE ..age <= <age>");
//! ```

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
