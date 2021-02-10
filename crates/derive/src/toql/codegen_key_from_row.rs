use crate::sane::FieldKind;
use proc_macro2::{Span, TokenStream};
use syn::Ident;
use std::collections::HashSet;

pub(crate) struct CodegenKeyFromRow<'a> {
    rust_struct: &'a crate::sane::Struct,

    forward_key_columns: usize,
    deserialize_key: Vec<TokenStream>,
    forward_join_key: Vec<TokenStream>,
    regular_types: HashSet<syn::Ident>,
    join_types: HashSet<syn::Ident>,
}

impl<'a> CodegenKeyFromRow<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenKeyFromRow {
        CodegenKeyFromRow {
            rust_struct: &toql,
            forward_key_columns: 0,
            deserialize_key: Vec::new(),
            forward_join_key: Vec::new(),
            regular_types: HashSet::new(),
            join_types: HashSet::new(),
        }
    }

    pub fn add_key_deserialize(
        &mut self,
        field: &crate::sane::Field,
    ) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_field_ident = &field.rust_field_ident;
        
         let error_field = format!(
                    "{}Key::{}",
                    &self.rust_struct.rust_struct_ident, rust_field_name
                );

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    return Ok(());
                }
                self.regular_types.insert(field.rust_base_type_ident.to_owned());
               
               /*  let increment = if self.deserialize_key.is_empty() {
                    quote!(*i)
                } else {
                    quote!({
                        *i = *i + 1;
                        *i
                    })
                }; */
                self.deserialize_key.push(quote!(
                    #rust_field_ident: {
                                    toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )?
                                            .ok_or(toql::error::ToqlError::DeserializeError(#error_field.to_string(), String::from("Deserialization stream is invalid: Expected selected field but got unselected.")))?
                                            
                    }
                ));
                self.forward_key_columns = self.forward_key_columns + 1;
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }

            self.join_types.insert(field.rust_base_type_ident.to_owned());
                // Impl key from result row
                self.forward_join_key.push(quote!(
                   *i = < #rust_type_ident > ::skip(*i);
                ));

              
                self.deserialize_key.push(quote!(
                    
                    #rust_field_ident: {
                         << #rust_type_ident as toql :: keyed :: Keyed > :: Key >:: from_row(row, i, iter)?
                                .ok_or(toql::error::ToqlError::ValueMissing(#error_field.to_string()))?
                    }
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

impl<'a> quote::ToTokens for CodegenKeyFromRow<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let _vis = &self.rust_struct.rust_struct_visibility;
        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;

        let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
      
        let deserialize_key = &self.deserialize_key;
        let regular_types = &self.regular_types.iter().map(|k| quote!( #k :toql::from_row::FromRow<R,E>, )).collect::<Vec<_>>();
        let join_types = &self.join_types.iter().map(|k| quote!( 
            #k :  toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
            <#k as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
            )).collect::<Vec<_>>();
        
        let regular_types_ref = regular_types.clone();
        let join_types_ref = join_types.clone();

       
        let key = quote! {
                
                    impl<R,E> toql::from_row::FromRow<R, E> for #struct_key_ident 
                    where  E: std::convert::From<toql::error::ToqlError>,
                     #(#regular_types)*
                     #(#join_types)*
                    
                    {
                                    
                            #[allow(unused_variables, unused_mut)]
                            fn from_row<'a, I> ( mut row : &R , i : &mut usize, mut iter: &mut I)
                                -> std::result:: Result < Option<#struct_key_ident>, E> 
                                where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> {

                                Ok ( Some(#struct_key_ident{
                                    #(#deserialize_key),*
                                }))
                            }

                }

                  /*   impl<R,E> toql::from_row::FromRow<R, E> for &#struct_key_ident 
                    where  E: std::convert::From<toql::error::ToqlError>,
                     #(#regular_types_ref)*
                     #(#join_types_ref)*
                    
                    {
                            fn from_row<'a, I> ( mut row : &R , i : &mut usize, mut iter: &mut I) {
                                <#struct_key_ident as toql::from_row::FromRow<R, E>>::from_row(row, usize, iter)
                            }
                    } */

        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
