//! The `#[derive(Toql)]` creates all the boilerplate code to make the ✨ happen.
//! Using the derive is the easy. However beware that the generated code size can become large 
//! as it's about ~3K lines of code for a medium `struct`.
//! 
//! For a bigger project, you are strongly advised to create a cargo workspace and
//! to put your Toql derived structs into a separate crate to reduce compile time.
//! This will pay off once your database model stabilizes.
//!
//! The `#[derive(ToqlEnum)]` must be added on enums to implement deserialization and conversion.
//! Notice that `ToqlEnum` requires enums to have implementations for the `ToString` and `FromStr` traits.

#![recursion_limit = "1024"]

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
/// Derive to deserialize enums.
#[proc_macro_derive(ToqlEnum)]
pub fn toql_enum_derive(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let name_string = name.to_string();
    let gen = quote! {
                       impl<R, E> toql::from_row::FromRow<R, E> for #name
                    where String :toql::from_row::FromRow<R, E>,
                    Self: std::str::FromStr,
                    E: std::convert::From<toql::error::ToqlError>,
                    {
                        fn forward<'a, I>(iter: &mut I) -> Result<usize, E>
                        where
                            I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
                        {
                            if  iter.next()
                                .ok_or(toql::error::ToqlError::DeserializeError(
                                        toql::deserialize::error::DeserializeError::StreamEnd))?
                                .is_selected() {
                                Ok(1)
                            } else {
                                Ok(0)
                            }
                        }
                        fn from_row<'a, I>(
                            row: &R,
                            i: &mut usize,
                            iter: &mut I,
                        ) -> std::result::Result<Option< #name >, E>
                        where
                            I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
                        {
                            let s: Option<String> = toql::from_row::FromRow::<R, E>::from_row(row, i, iter)?;
                            if let Some(s) = s {
                                let t = <Self as std::str::FromStr>::from_str(s.as_str())
                                    .map_err(|e|toql::error::ToqlError::DeserializeError(
                                        toql::deserialize::error::DeserializeError::ConversionFailed( #name_string .to_string(), e.to_string()))
                                    )?;
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
                                    .map_err(|e|toql::error::ToqlError::DeserializeError(
                                         toql::deserialize::error::DeserializeError::ConversionFailed(#name_string .to_string(), e.to_string())))?;
                                Ok(t)
                        } else {
                            Err(toql::error::ToqlError::DeserializeError(
                                toql::deserialize::error::DeserializeError::ConversionFailed(#name_string .to_string(),"Requires string argument.".to_string())))
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


#[test]
fn derive_fields() {
    use annot::Toql;
    let input = r#"
    #[toql(auto_key = true)]
    struct User {
        #[toql(key)]
        id: u64,
        name: String
    }"#;
    
    // Parse valid Rust syntax
    let m  = syn::parse_str::<syn::DeriveInput>(input);
    println!("{:?}", &m);
    assert_eq!(m.is_ok(), true);

    let m = m.unwrap();
     // Parse struct attributes, visibilty, generics and data
    let derive = Toql::from_derive_input(&m);
       
    println!("{:?}", &derive);
    assert!(codegen.is_ok())
   // assert!(matches!(m,  FieldsMacro::FieldList{..}));
   /*  if let FieldsMacro::FieldList{query, struct_type} = m {
        let f = fields_macro::parse(&query, struct_type); 
        assert_eq!(f.is_ok(), true);
    } */
}