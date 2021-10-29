//! The `paths!` macro compiles a list of Toql field names into program code. 
//! Any syntax errors or wrong path names will show up at compile time.
//!
//! Wrong path names are detected because the `paths!` macro uses the 
//! query builder functions that are genereated by the Toql derive.
//!
//! ### Example
//! Assume a `struct User` with a joined `address`.ok_or(none_error!())?
//! ```rust, ignore
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


#[test]
fn valid_path_list() {
    use paths_macro::PathsMacro;
    let input = "User, \"prop1, prop2\" ";
    
    let m  = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

   let m = m.unwrap();
    assert!(matches!(m, PathsMacro::PathList{..}));
    if let PathsMacro::PathList{query, struct_type} = m {
        let f = paths_macro::parse(&query, struct_type); 
        assert_eq!(f.is_ok(), true);
    }
}
#[test]
fn valid_top() {
    use paths_macro::PathsMacro;
    let input = "top";
    
    let m  = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

   let m = m.unwrap();
    assert!(matches!(m,  PathsMacro::Top));
}
#[test]
fn invalid_mixed_case_top() {
    use paths_macro::PathsMacro;
    let input = "Top";
    
    let m :syn::Result<PathsMacro> = syn::parse_str(input);
    assert_eq!(m.is_ok(), false);
}
#[test]
fn missing_pathlist() {
    use paths_macro::PathsMacro;
    let input = "User";
    
    let m :syn::Result<PathsMacro> = syn::parse_str(input);
    assert_eq!(m.is_ok(), false);
}

#[test]
fn invalid_pathlist() {
    use paths_macro::PathsMacro;
    let input = "User, \"prop1 prop2\" "; // missing comma
    
    let m = syn::parse_str(input);
    assert_eq!(m.is_ok(), true);

    let m = m.unwrap();
    assert!(matches!(m,  PathsMacro::PathList{..}));
    if let PathsMacro::PathList{query, struct_type} = m {
        let f = paths_macro::parse(&query, struct_type); 
        assert_eq!(f.is_err(), true);
    }
}