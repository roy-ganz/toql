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
mod codegen;

#[cfg(feature = "mysqldb")]
mod codegen_mysql;

mod util;

#[proc_macro_derive(Toql, attributes(toql))]
pub fn toql_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
   
    let generated_result = annot::Toql::from_derive_input(&ast);

    match generated_result {
        Ok(gen) =>  TokenStream::from(quote!(#gen)),
        Err(error) => TokenStream::from(error.write_errors())
    }
}
