/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
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
        match &field.kind {
            FieldKind::Join(join_attrs) => {
               self.dispatch_predicate_code.push(
                   quote!(
                        #toql_field_name => {
                            <#rust_type_ident as toql::tree::TreePredicate>::predicate(&#rust_field_ident ,&p, &mut predicate)
                        }
                )
               );
            }
            FieldKind::Merge(_) => {
                self.dispatch_predicate_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in #rust_field_ident {
                            <#rust_type_ident as toql::tree::TreePredicate>::predicate(&f, &p, &mut predicate)
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
       
       let mods =  quote! {
                impl toql::tree::TreePredicate for #struct_ident {
                    fn predicate<'a>(&self,  descendents: &toql::query::field_path::Descendents<'a>, 
                            predicate: &mut toql::sql::Sql) -> toql::error::Result<()> {
                         match descendents.next() {
                               Some(d) => match d {
                                   // #(#dispatch_predicate_code*),
                                   #(#dispatch_predicate_code)* ,
                                   f @ _ => return Err(toql::error::ToqlError::SqlBuilderError(toql::sql_builder::SqlBuilderError::FieldMissing(f.to_string())));
                               },
                               None => {
                                        // TODO Sql Expr because of alias
                                         predicate.push(toql::key::predicate_sql(&[self.try_get_key()?]));
                                         predicate.push_str(" OR ");
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
