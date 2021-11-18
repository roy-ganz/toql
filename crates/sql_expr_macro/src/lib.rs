//! The `sql_expr!` macro compiles an SQL expression into program code.
//!
//! ### Example
//! ```rust, ignore
//! use toql_sql_expr_macro::sql_expr;
//!
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

#[test]
fn literal_and_alias() {
    use sql_expr_macro::SqlExprMacro;
    let input = "\"SELECT ..id FROM Table .. JOIN OtherTable ...\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let SqlExprMacro { query, arguments } = m.unwrap();
    let f = sql_expr_macro::parse(&query, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "{ let mut t = toql :: sql_expr :: SqlExpr :: new ( ) ; \
        t . extend ( toql :: sql_expr :: SqlExpr :: from ( vec ! [ \
            toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \"SELECT \" ) ) , \
            toql :: sql_expr :: SqlExprToken :: SelfAlias , \
            toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \".id FROM Table \" ) ) , \
            toql :: sql_expr :: SqlExprToken :: SelfAlias , \
            toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \" JOIN OtherTable \" ) ) , \
            toql :: sql_expr :: SqlExprToken :: OtherAlias ] ) ) ; t }");
}
#[test]
fn placeholders_and_aux_params() {
    use sql_expr_macro::SqlExprMacro;
    let input = "\"SELECT ?, <aux_parm>\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let SqlExprMacro { query, arguments } = m.unwrap();
    let f = sql_expr_macro::parse(&query, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "{ let mut t = toql :: sql_expr :: SqlExpr :: new ( ) ; \
    t . extend ( toql :: sql_expr :: SqlExpr :: from ( vec ! [ \
        toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \"SELECT \" ) ) , \
        toql :: sql_expr :: SqlExprToken :: UnresolvedArg , \
        toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \", \" ) ) , \
        toql :: sql_expr :: SqlExprToken :: AuxParam ( String :: from ( \"aux_parm\" ) ) ] ) ) ; t }");
}
#[test]
fn quotes() {
    use sql_expr_macro::SqlExprMacro;
    let input = "\"SELECT '''?', '<aux_parm>'\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let SqlExprMacro { query, arguments } = m.unwrap();
    let f = sql_expr_macro::parse(&query, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "{ let mut t = toql :: sql_expr :: SqlExpr :: new ( ) ; \
    t . extend ( toql :: sql_expr :: SqlExpr :: from ( vec ! [ \
        toql :: sql_expr :: SqlExprToken :: Literal ( String :: from ( \"SELECT \'\'\'?\', \'<aux_parm>\'\" ) ) ] ) ) ; t }");
}
