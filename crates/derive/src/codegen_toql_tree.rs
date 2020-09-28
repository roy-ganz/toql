/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) struct GeneratedToqlTree<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    
    dispatch_predicate_code: Vec<TokenStream>,
   
}

impl<'a> GeneratedToqlTree<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlTree {
        GeneratedToqlTree {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            
            dispatch_predicate_code: Vec::new(),
        }
    }

    pub(crate) fn add_tree_traits(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_type_ident = &field.rust_type_ident;
        let toql_field_name= &field.toql_field_name;

    

        // Handle key predicate and parameters
         let unwrap = match field.number_of_options {
                    1 => quote!(.as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                    0 => quote!(.as_ref()),
                    _ => quote!(.as_ref().unwrap().as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                };


        match &field.kind {
            FieldKind::Join(join_attrs) => {

               
               self.dispatch_predicate_code.push(
                   quote!(
                       self. #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            predicate(&#rust_field_ident # unwrap ,&mut descendents, predicate, args)?
                        }
                )
               );
            }
            FieldKind::Merge(_) => {
                self.dispatch_predicate_code.push(
                   quote!(
                      self. #toql_field_name => {
                        for f in & #rust_field_ident #unwrap {
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            predicate(f, &mut descendents, predicate, args)?
                        }
                       }
                )
               );
            }
            _ => {
                
            }
        };


    }
}
impl<'a> quote::ToTokens for GeneratedToqlTree<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let dispatch_predicate_code = &self.dispatch_predicate_code;
        let struct_key_ident = Ident::new(&format!("{}Key", &self.struct_ident), Span::call_site());
       
       let mods =  quote! {
                impl toql::tree::tree_predicate::TreePredicate for #struct_ident {
                    fn predicate<'a>(&self,  mut descendents: &mut toql::query::field_path::Descendents<'a>, 
                            mut predicate: &mut toql::sql_expr::SqlExpr,  mut args: &mut Vec<toql::sql_arg::SqlArg>) -> toql::error::Result<()> {
                         match descendents.next() {
                               Some(d) => match d.as_str() {
                                   // #(#dispatch_predicate_code*),
                                   #(#dispatch_predicate_code),* 
                                   f @ _ => {
                                        return Err(toql::error::ToqlError::SqlBuilderError(
                                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string())));
                                    }
                               },
                               None => {
                                        // TODO Sql Expr because of alias
                                        let key = toql::key::Keyed::try_get_key(self) ? ; 
                                        if  #struct_key_ident ::columns() . len() == 1 {
                                                 predicate.push_literal(" KEY WIth IN ".to_string());
                                        } else {
                                            if !predicate.is_empty(){
                                                predicate.push_literal(" OR ");
                                            }
                                            predicate.push_literal(" KEY WIth AND ".to_string());
                                         
                                        }
                               }
                        } 
                        Ok(())
                    }
               }
        };

        log::debug!(
            "Source code for `{}`:\n{}",
            self.struct_ident,
            mods.to_string()
        );
        tokens.extend(mods);
    }
}
