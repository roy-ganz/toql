//! The `role_expr!` macro compiles a role expression into program code.
//!
//! ### Example
//! ```rust, ignore
//! use toql_role_expr_macro::role_expr;
//!
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

#[test]
fn role_names() {
    use role_expr_macro::RoleExprMacro;
    let input = "\"role1, role_1\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let RoleExprMacro { query } = m.unwrap();
    let f = role_expr_macro::parse(&query);
    assert_eq!(f.is_ok(), true);

    assert_eq!(
        f.unwrap().to_string(),
        "toql :: role_expr :: RoleExpr :: And ( \
            Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role1\" . to_string ( ) ) ) , \
            Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role_1\" . to_string ( ) ) ) )"
    );
}
#[test]
fn concatenation() {
    use role_expr_macro::RoleExprMacro;
    let input = "\"role1, role2; !role3\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let RoleExprMacro { query } = m.unwrap();
    let f = role_expr_macro::parse(&query);
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: role_expr :: RoleExpr :: Or ( \
            Box :: new ( toql :: role_expr :: RoleExpr :: And ( \
                Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role1\" . to_string ( ) ) ) , \
                Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role2\" . to_string ( ) ) ) ) ) , \
            Box :: new ( toql :: role_expr :: RoleExpr :: Not ( \
                Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role3\" . to_string ( ) ) ) ) ) )");
}
#[test]
fn parenthesis() {
    use role_expr_macro::RoleExprMacro;
    let input = "\"(((role1); role2), role3)\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let RoleExprMacro { query } = m.unwrap();
    let f = role_expr_macro::parse(&query);
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: role_expr :: RoleExpr :: And ( \
            Box :: new ( toql :: role_expr :: RoleExpr :: Or ( \
                Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role1\" . to_string ( ) ) ) , \
                Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role2\" . to_string ( ) ) ) ) ) , \
            Box :: new ( toql :: role_expr :: RoleExpr :: role ( \"role3\" . to_string ( ) ) ) )");
}
