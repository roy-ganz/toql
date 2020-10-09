/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget, MergeColumn};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) struct GeneratedToqlTree<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    
    dispatch_predicate_code: Vec<TokenStream>,
    dispatch_merge_key_code: Vec<TokenStream>,
    merge_columns_code: Vec<TokenStream>,
    merge_predicate_code: Vec<TokenStream>,

    index_type_bounds: Vec<TokenStream>,
    dispatch_index_code: Vec<TokenStream>,
    index_code: Vec<TokenStream>
   
}

impl<'a> GeneratedToqlTree<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlTree {
        GeneratedToqlTree {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            
            dispatch_predicate_code: Vec::new(),
            dispatch_merge_key_code: Vec::new(),
            merge_columns_code: Vec::new(),
            merge_predicate_code: Vec::new(),

            index_type_bounds: Vec::new(),
            dispatch_index_code: Vec::new(),
            index_code: Vec::new(),
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
                    0 => quote!(),
                    _ => quote!(.as_ref().unwrap().as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                };


        match &field.kind {
            FieldKind::Join(join_attrs) => {

               
               self.dispatch_predicate_code.push(
                   quote!(
                       self. #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            predicate(&#rust_field_ident # unwrap ,&mut descendents, &field, key_expr)?
                        }
                )
               );

               self.dispatch_index_code.push(
                   quote!(
                       self. #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_index::TreeIndex>::
                            index(&#rust_field_ident # unwrap ,&mut descendents, &field, index)?
                        }
                )
               );
            }
            FieldKind::Merge(merge) => {
                self.dispatch_index_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in &self. #rust_field_ident #unwrap {
                            <#rust_type_ident as toql::tree::tree_index::TreeIndex>::
                            index(f, &mut descendents, &field, index)?
                        }
                       }
                )
               );

                self.dispatch_predicate_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in &self. #rust_field_ident #unwrap {
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            predicate(f, &mut descendents, &field, predicate)?
                        }
                       }
                )
               );
                self.dispatch_merge_key_code.push(
                   quote!(
                       #toql_field_name => {
                            <#rust_type_ident as toql::tree::tree_keys::TreeKeys>::
                            keys(&mut descendents, field, key_expr)?
                        }                       
                )
               );
               let mut columns_merge = Vec::new();
               for c  in &merge.columns {
                   //let this_col = c.this;
                  match &c.other {
                       MergeColumn::Aliased(a) => {   columns_merge.push( quote!(
                           key_expr.push_literal(#a);
                           ));}
                       MergeColumn::Unaliased(u) => {  columns_merge.push(quote!(
                           key_expr.push_self_alias();
                              key_expr.push.push_literal(".");
                                 key_expr.push.push_literal(#u);

                           ));}
                   }
                    columns_merge.push( quote!(
                        key_expr.push_literal(", ");
                        ));

               }
                self.merge_columns_code.push(
                   quote!(
                       #toql_field_name => {
                            // Primary key
                            for col in <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns() {
                                key_expr.push_self_alias();
                                key_expr.push_literal(".");
                                key_expr.push_alias(col);
                                key_expr.push_literal(", ");
                            }
                           #(#columns_merge)*
                       }
                )
               );

               let mut columns_code : Vec<TokenStream> = Vec::new();
               for c in &merge.columns {
                   columns_code.push(match &c.other {
                       MergeColumn::Aliased(a) => { quote!( columns.push(  toql :: sql_expr :: PredicateColumn::Literal(#a .to_owned())); )}
                       MergeColumn::Unaliased(a) => {quote!( columns.push(  toql :: sql_expr :: PredicateColumn::SelfAliased(#a .to_owned())); )}
                   });
               }

                
                self.merge_predicate_code.push(
                   quote!(
                       #toql_field_name => {
                                let key = << Self as toql :: key :: Keyed >::try_get_key(&self)?;
                                let params =<< Self as toql :: key :: Keyed > :: Key as toql :: key :: Key > ::params(&key);
                                let mut columns :Vec<toql::sql_expr::PredicateColumn> = Vec::new();
                                #(#columns_code)*
                                predicate.push_predicate( columns, params); 
                        },
                          
                )
               );

                let type_key_ident = Ident::new(&format!("{}Key", &field.rust_type_name), Span::call_site());
                let struct_ident = self.struct_ident;
                self.index_type_bounds.push(quote!(
                    #type_key_ident : toql :: from_row :: FromRow < R >, 
                    <#struct_ident as toql :: from_row :: FromRow < R >> :: Error : std::convert::From< < #type_key_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));
               self.index_code.push(
                    quote!(
                        #toql_field_name => {
                            let fk = #type_key_ident ::from_row_with_index(&row, &mut i, &mut iter)?;
                            fk.hash(&mut s);
                            },
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
        let dispatch_merge_key_code = &self.dispatch_merge_key_code;

        let merge_columns_code = &self.merge_columns_code;
        let merge_predicate_code = &self.merge_predicate_code;

        let index_type_bounds = &self.index_type_bounds;
        let dispatch_index_code = &self.dispatch_index_code;
        let index_code = &self.index_code;

        let struct_key_ident = Ident::new(&format!("{}Key", &self.struct_ident), Span::call_site());
       
       let mods =  quote! {
                impl toql::tree::tree_predicate::TreePredicate for #struct_ident {
                    fn predicate<'a>(&self,  mut descendents: &mut toql::query::field_path::Descendents<'a>, 
                    field: &str,
                            mut predicate: &mut toql::sql_expr::SqlExpr) 
                            ->std::result::Result<(), toql::error::ToqlError> {
                         match descendents.next() {
                               Some(d) => match d.as_str() {
                                   #(#dispatch_predicate_code),* 
                                   f @ _ => {
                                        return Err(
                                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                    }
                               },
                               None => {
                                    match field {
                                     #(#merge_predicate_code),* 
                                     f @ _ => {
                                        return Err(
                                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                    }
                                    }
                                  
                               }
                        } 
                        Ok(())
                    }
               }

                 
                impl toql::tree::tree_keys::TreeKeys for #struct_ident
                {
                    fn keys<'a>(
                        mut descendents: &mut toql::query::field_path::Descendents<'a>,
                        field: &str,
                        key_expr: &mut toql::sql_expr::SqlExpr,
                    ) -> Result<(),toql::sql_builder::sql_builder_error::SqlBuilderError> {

                            match descendents.next() {
                            
                                Some(d) => { 
                                    match d.as_str() {
                                        #(#dispatch_merge_key_code),* 
                                        f @ _ => {
                                            return Err(
                                               toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()));
                                        }
                                    }
                                },
                                None => {
                                    // Private key
                                    /* for col in <#struct_key_ident as toql::key::Key>::columns() {
                                        key_expr.push_self_alias();
                                        key_expr.push_literal(".");
                                        key_expr.push_alias(col);
                                        key_expr.push_literal(", ");
                                    } */
                                     match field {
                                        #(#merge_columns_code),*
                                        
                                        "" => {
                                            for col in <#struct_key_ident as toql::key::Key>::columns() {
                                                key_expr.push_self_alias();
                                                key_expr.push_literal(".");
                                                key_expr.push_alias(col);
                                                key_expr.push_literal(", ");
                                            }
                                        }, 
                                        f @ _ => {
                                            return Err(
                                                toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()));
                                        }
                                    }
                                   
                                    key_expr.pop(); // Remove final ", "
                                }
                        }
                        Ok(())  
                    }  
                         
                }

  

                impl<R> toql::tree::tree_index::TreeIndex<R> for #struct_ident 
                where Self: toql::from_row::FromRow<R>,
                #struct_key_ident : toql :: from_row :: FromRow < R >,
                <#struct_ident as toql :: from_row :: FromRow < R >> :: Error : std::convert::From< <#struct_key_ident as toql :: from_row :: FromRow < R >> :: Error>,
                <Self as toql :: from_row :: FromRow < R >>::Error: std::convert ::From<toql :: sql_builder :: sql_builder_error ::  SqlBuilderError>,
                #(#index_type_bounds)*
  
                {
                    fn index<'a, I>( descendents: &toql::query::field_path::Descendents<'a>, field: &str, 
                                rows: I, index: &mut HashMap<u64,Vec<usize>>) 
                        -> std::result::Result<(), <Self as toql::from_row::FromRow<R>>::Error>
                         where I: IntoIterator<Item=R>
                         {
                          
                        use toql::from_row::FromRow;
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        use std::collections::hash_map::DefaultHasher;

                      match descendents.next() {
                            
                                Some(d) => { 
                                    match d.as_str() {
                                        #(#dispatch_index_code),* 
                                        f @ _ => {
                                            return Err(
                                               toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()));
                                        }
                                    }
                                },
                                None => {
                                   
                                 
                                     

                                        let mut  i= 0;
                                        let mut iter = std::iter::repeat(&true);
                                        i = <#struct_ident as toql :: from_row :: FromRow<R>>::skip(i);
                                        for (n, row) in rows.into_iter().enumerate() {
                                            let mut s = DefaultHasher::new();
                                            let pk =  #struct_key_ident ::from_row_with_index(&row, &mut i, &mut iter)?;
                                            pk.hash(&mut s);
                                            let pk_hash= s.finish();

                                            let mut s = DefaultHasher::new();
                                            let fh_hash = match field {
                                               #(#index_code)*
                                               
                                                f @ _ => {
                                                    return Err(
                                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()));
                                                }

                                            };
                                            let fk_hash =  s.finish();
                                    

                                            index.entry(pk_hash)
                                            .and_modify(|h| h.push(n))
                                            .or_insert(vec![n]);
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
