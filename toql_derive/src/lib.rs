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
mod codegen_toql_key;
mod codegen_toql_mapper;
mod codegen_toql_delup;
mod codegen_toql_query_builder;

#[cfg(feature = "mysqldb")]
mod codegen_mysql_load;

#[cfg(feature = "mysqldb")]
mod codegen_mysql_select;

#[cfg(feature = "mysqldb")]
mod codegen_mysql_insert;

mod sane;
mod util;


#[proc_macro_derive(ToqlFilterArg)]
pub fn filter_arg_derive(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
    let ast = parse_macro_input!(input as DeriveInput);
    //let gen = impl_filter_arg_derive(&ast);
    let name = &ast.ident;
    let gen = quote! {
				impl toql::query::FilterArg for  &#name {
					fn to_sql(&self) -> String {
						 self.to_string().to_sql()
					}
				}
    };
     log::debug!("Source code for `{}`:\n{}", &name, gen.to_string());
    TokenStream::from( gen)
}


/* fn impl_filter_arg_derive(ast: &syn::DeriveInput) ->TokenStream {
    let name = &ast.ident;

    quote! {
				impl toql::query::FilterArg for  &#name {
					fn to_sql(&self) () -> String {
						 self.to_string().to_sql()
					}
				}
    }
} */

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
