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

extern crate proc_macro;

extern crate syn;

extern crate heck;
#[macro_use]
extern crate darling;

#[macro_use]
extern crate quote;

use syn::{parse_macro_input, DeriveInput};

use darling::FromDeriveInput;

use proc_macro::TokenStream;

mod annot;

mod toql;


mod sane;
mod util;

mod string_set;

#[proc_macro_derive(ToqlEnum)]
pub fn toql_enum_derive(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
    let ast = parse_macro_input!(input as DeriveInput);
    //let gen = impl_filter_arg_derive(&ast);
    let name = &ast.ident;
    let gen = quote! {
                       impl<R, E> toql::from_row::FromRow<R, E> for #name 
                    where String :toql::from_row::FromRow<R, E>, 
                    Self: std::str::FromStr, 
                    E: std::convert::From<toql::error::ToqlError>,
                    {
                        fn from_row<'a, I>(
                            row: &R,
                            i: &mut usize,
                            iter: &mut I,
                        ) -> std::result::Result<Option<ConfigurationType>, E>
                        where
                            I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
                        {
                            let s: Option<String> = toql::from_row::FromRow::<R, E>::from_row(row, i, iter)?;
                            if let Some(s) = s {
                                let t = <Self as std::str::FromStr>::from_str(s.as_str())
                                    .map_err(|e|toql::error::ToqlError::DeserializeError( #name .to_string(), e.to_string()))?;
                                Ok(Some(t))

                            } else {
                                Ok(None)
                            }

                        }
                    }
                    impl std::convert::TryFrom<&toql::sql_arg::SqlArg> for #name 
                    where Self: std::str::FromStr {

                        type Error =  toql::error::ToqlError;
                        fn try_from(t: &toql::sql_arg::SqlArg) -> Result<Self, Self::Error> {
                        if let toql::sql_arg::SqlArg::Str(s) = t {
                                let t = <Self as std::str::FromStr>::from_str(s.as_str())
                                    .map_err(|e|toql::error::ToqlError::DeserializeError( #name.to_string(), e.to_string()))?;
                                Ok(t)
                        } else {
                            Err(toql::error::ToqlError::DeserializeError(#name.to_string(),"Requires string argument.".to_string()))  
                        }
                        }
                    }

                    impl From<#name> for toql::sql_arg::SqlArg {
                        fn from(t: #name) -> Self {
                            toql::sql_arg::SqlArg::Str(t.to_string())
                        }
                    }
                     impl From<&#name> for toql::sql_arg::SqlArg {
                        fn from(t: &#name) -> Self {
                            toql::sql_arg::SqlArg::Str(t.to_owned().to_string())
                        }
                    }
    };
    log::debug!("Source code for `{}`:\n{}", &name, gen.to_string());
    TokenStream::from(gen)
}


/// Derive to add Toql functionality to your struct.
#[proc_macro_derive(Toql, attributes(toql))]
pub fn toql_derive(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
    let ast = parse_macro_input!(input as DeriveInput);

    let generated_result = annot::Toql::from_derive_input(&ast);

    match generated_result {
        Ok(gen) => TokenStream::from(quote!(#gen)),
        Err(error) => TokenStream::from(error.write_errors()),
    }
}


