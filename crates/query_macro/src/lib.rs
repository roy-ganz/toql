//! The `query!` macro compiles a Toql query into program code.
//! Any syntax errors, wrong paths or field names will show up at compile time.
//! Wrong paths or field names are detected because the query! macro uses the
//! query builder functions that are genereated by the Toql derive.
//!
//! ### Example
//! ```rust, ignore
//! use toql_query_macro::query;
//! use toql_fields_macro::fields;
//!
//! #[derive(Toql)]
//! struct User
//!     #[toql(key)]
//!     id: u64,
//!     name: String,
//!     #[toql(join())]
//!     address: Address
//! }
//!
//! #[derive(Toql)]
//! struct Address
//!     #[toql(key)]
//!     id: u64,
//!     street: String
//! }
//!
//! let f = query!(User, "id eq ?, address_street", 42);
//! ```

#![recursion_limit = "512"]

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::parse_macro_input;

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

#[test]
fn fields() {
    use query_macro::QueryMacro;
    let input = "User, \"prop1, level2_prop2, *, level2_*\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop1 ( ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#level2 ( ) . r#prop2 ( ) ) \
                . and ( toql :: query_path :: QueryPath :: wildcard ( < User as toql :: query_fields :: QueryFields > :: fields ( ) ) ) \
                . and ( toql :: query_path :: QueryPath :: wildcard ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#level2 ( ) ) )");
}

#[test]
fn filters() {
    use query_macro::QueryMacro;
    let input = "User, \"prop eq 1.5, prop eqn, prop ne 1, prop nen, prop gt 1, prop ge 1,\
                        prop lt 1, prop le 1, prop lk 'ABC', \
                        prop in 1 2 3, prop out 1 2 3, prop bw 1 10,\
                        prop fn cst 'A' 'B' 'C'\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eq ( 1.5f64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eqn ( ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . ne ( 1u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . nen ( ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . gt ( 1u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . ge ( 1u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . lt ( 1u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . le ( 1u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . lk ( \"ABC\" ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) \
                . ins ( & [ 1u64 , 2u64 , 3u64 ] ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . out ( & [ 1u64 , 2u64 , 3u64 ] ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . bw ( 1u64 , 10u64 ) ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) \
                    . fnc ( \"CST\" , & [ \"A\" , \"B\" , \"C\" ] ) )");
}

#[test]
fn selections() {
    use query_macro::QueryMacro;
    let input = "User, \"$std, $level1_std\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
        . and ( toql :: query_path :: QueryPath :: selection ( < User as toql :: query_fields :: QueryFields > :: fields ( ) , \"std\" ) ) \
        . and ( toql :: query_path :: QueryPath :: selection ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#level1 ( ) , \"std\" ) )");
}
#[test]
fn predicates() {
    use query_macro::QueryMacro;
    let input = "User, \"@pred, @pred 'A' 5, @level1_pred\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(),"toql :: query :: Query :: < User > :: new ( ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#pred ( ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#pred ( ) . are ( & [ \"A\" , 5u64 ] ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#level1 ( ) . r#pred ( ) )");
}

#[test]
fn order_and_hiding() {
    use query_macro::QueryMacro;
    let input = "User, \"+prop, +2prop, -prop, -.prop, +.prop\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . asc ( 1u8 ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . asc ( 2u8 ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . desc ( 1u8 ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . desc ( 1u8 ) . hide ( ) ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . asc ( 1u8 ) . hide ( ) )");
}
#[test]
fn arguments() {
    use query_macro::QueryMacro;
    let input = "User, \"prop; prop eq ?; prop eq ?\", 5, \"ABC\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());

    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
    . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) ) \
    . or ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eq ( 5 ) ) \
    . or ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eq ( \"ABC\" ) )");
}
#[test]
fn parenthesis() {
    use query_macro::QueryMacro;
    let input = "User, \"(((prop; prop eq ?)); (prop eq ?))\", 5, \"ABC\"";

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());

    assert_eq!(f.is_ok(), true);

    assert_eq!(f.unwrap().to_string(), "toql :: query :: Query :: < User > :: new ( ) \
    . and_parentized ( toql :: query :: Query :: < User > :: new ( ) \
        . and_parentized ( toql :: query :: Query :: < User > :: new ( ) \
            . and_parentized ( toql :: query :: Query :: < User > :: new ( ) \
                . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) ) \
                . or ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eq ( 5 ) ) ) ) \
        . or_parentized ( toql :: query :: Query :: < User > :: new ( ) \
            . and ( < User as toql :: query_fields :: QueryFields > :: fields ( ) . r#prop ( ) . eq ( \"ABC\" ) ) ) )");
}

#[test]
fn too_many_arguments() {
    use query_macro::QueryMacro;
    let input = "User, \"*\", x"; // {} or is missing for argument x

    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);
    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_err(), true);
}

#[test]
fn missing_arguments() {
    use query_macro::QueryMacro;
    let input = "User, \"*,?\""; // missing argument for placeholder
    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);
    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_err(), true);

    let input = "User, \"*,{}\""; // missing argument for query
    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);
    let QueryMacro {
        query,
        struct_type,
        arguments,
    } = m.unwrap();
    let f = query_macro::parse(&query, struct_type, &mut arguments.iter());
    assert_eq!(f.is_err(), true);
}
