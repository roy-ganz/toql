//! The `role_expr!` macro compiles a role expression into program code. 
//!
//! ### Example
//! ```rust
//! let f = role_expr!("admin;power_user");
//! ```

#![recursion_limit = "512"]

extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse_macro_input;

use proc_macro::TokenStream;

mod role_expr_macro;

#[proc_macro]
pub fn role_expr(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
                                    // eprintln!("{:?}", input);

    let ast = parse_macro_input!(input as role_expr_macro::RoleExprMacro);

    let gen = role_expr_macro::parse(&ast.query);

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
