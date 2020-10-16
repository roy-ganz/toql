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
    index_code: Vec<TokenStream>,
    merge_type_bounds: Vec<TokenStream>,
    dispatch_merge_code: Vec<TokenStream>,
    merge_code: Vec<TokenStream>
   
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
            merge_type_bounds: Vec::new(),
            dispatch_merge_code: Vec::new(),
            merge_code: Vec::new(),
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

                let refer = match field.number_of_options {
                    0 => quote!(&),
                    _ => quote!(),
                };
                 let unwrap_mut = match field.number_of_options {
                    1 => quote!(.as_mut().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                    0 => quote!(),
                    _ => quote!(.as_mut().unwrap().as_mut().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                };

                let refer_mut = match field.number_of_options {
                    0 => quote!(&mut),
                    _ => quote!(),
                };


        match &field.kind {
            FieldKind::Join(_join_attrs) => {

               
               self.dispatch_predicate_code.push(
                   quote!(
                      #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            predicate(#refer  self. #rust_field_ident # unwrap ,&mut descendents, &field, predicate)?
                        }
                )
               );

               self.dispatch_index_code.push(
                   quote!(
                        #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_index::TreeIndex<$row_type,$error_type>>::
                            index(&mut descendents, &field,rows, row_offset, index)?
                        }
                )
               );
                self.index_type_bounds.push(quote!(
                    #rust_type_ident : toql :: from_row :: FromRow < R >, 
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

                self.dispatch_merge_code.push(
                   quote!(
                       #toql_field_name => {
                        
                            <#rust_type_ident as toql::tree::tree_merge::TreeMerge<$row_type,$error_type>>::
                            merge(#refer_mut self. #rust_field_ident #unwrap_mut, &mut descendents, &field, rows, row_offset, index, selection_stream)?
                    
                       }
                )
               ); 
                self.merge_type_bounds.push(quote!(
                    #rust_type_ident : toql :: from_row :: FromRow < R >, 
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

               
            }
            FieldKind::Merge(merge) => {
                self.dispatch_index_code.push(
                   quote!(
                       #toql_field_name => {
                             <#rust_type_ident as toql::tree::tree_index::TreeIndex<$row_type,$error_type>>::
                            index(&mut descendents, &field,rows, row_offset, index)?
                       }
                )
               );
                self.dispatch_merge_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in #refer_mut self. #rust_field_ident #unwrap_mut {
                            <#rust_type_ident as toql::tree::tree_merge::TreeMerge<$row_type,$error_type>>::
                            merge(f, &mut descendents, &field, rows, row_offset, index, selection_stream)?
                        }
                       }
                )
               );

                self.dispatch_predicate_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in #refer self. #rust_field_ident #unwrap {
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
                            /* for col in <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns() {
                                key_expr.push_self_alias();
                                key_expr.push_literal(".");
                                key_expr.push_alias(col);
                                key_expr.push_literal(", ");
                            } */
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
                                let key = < Self as toql :: key :: Keyed >::try_get_key(&self)?;
                                let params =<< Self as toql :: key :: Keyed > :: Key as toql :: key :: Key > ::params(&key);
                                let mut columns :Vec<toql::sql_expr::PredicateColumn> = Vec::new();
                                #(#columns_code)*
                                predicate.push_predicate( columns, params); 
                        },
                          
                )
               );

                let type_key_ident = Ident::new(&format!("{}Key", &field.rust_type_name), Span::call_site());
                let struct_ident = self.struct_ident;
                let struct_key_ident = Ident::new(&format!("{}Key", &struct_ident), Span::call_site());

                self.index_type_bounds.push(quote!(
                    #type_key_ident : toql :: from_row :: FromRow < R >, 
                    E : std::convert::From< < #type_key_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));
                self.merge_type_bounds.push(quote!(
                    #type_key_ident : toql :: from_row :: FromRow < R >, 
                    E : std::convert::From< < #type_key_ident as toql :: from_row :: FromRow < R >> :: Error>,
                    #rust_type_ident : toql :: from_row :: FromRow < R >, 
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

               self.index_code.push(
                    quote!(
                        #toql_field_name => {
                            let fk = #type_key_ident ::from_row_with_index(&row, &mut i, &mut iter)?;
                            fk.hash(&mut s);
                            },
                    )
                );
                let merge_push = if field.number_of_options > 0 {
                    quote!( if self. #rust_field_name .is_none() {
                            self. #rust_field_name = Some(Vec::new())};
                            self. #rust_field_name .as_mut().unwrap() .push(e); 
                    )
                } else {
                    quote!(self. #rust_field_name .push(e);)
                };

               self.merge_code.push(
                    quote!(
                       
                             #toql_field_name  => {
                                for row_number in row_numbers {
                                            let mut i = n;
                                            let mut iter = std::iter::repeat(&Select::Query);
                                            let row: & $row_type = &rows[*row_number];
                                            let fk = #struct_key_ident::from_row_with_index(&row, &mut i, &mut iter)?;
                                            if fk ==  pk {
                                                let mut i = 0;
                                                let mut iter = selection_stream.iter();
                                                let e = #rust_type_ident::from_row_with_index(&row, &mut i, &mut iter)?;
                                                #merge_push
                                            }
                                        }
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

        let merge_type_bounds = &self.merge_type_bounds;
        let dispatch_merge_code = &self.dispatch_merge_code;
        let merge_code = &self.merge_code;

        let struct_key_ident = Ident::new(&format!("{}Key", &self.struct_ident), Span::call_site());
        let macro_name_index = Ident::new(&format!("toql_tree_index_{}", &self.struct_ident), Span::call_site());
        let macro_name_merge = Ident::new(&format!("toql_tree_merge_{}", &self.struct_ident), Span::call_site());
       
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
                                        
                                       /*  "" => {
                                            for col in <#struct_key_ident as toql::key::Key>::columns() {
                                                key_expr.push_self_alias();
                                                key_expr.push_literal(".");
                                                key_expr.push_alias(col);
                                                key_expr.push_literal(", ");
                                            }
                                        },  */
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

               macro_rules! #macro_name_index {
                // `()` indicates that the macro takes no argument.
                ($row_type: ty, $error_type: ty) => {
                 impl toql::tree::tree_index::TreeIndex<$row_type, $error_type> for #struct_ident 
               /*  where Self: toql::from_row::FromRow<R>,
                #struct_key_ident : toql :: from_row :: FromRow < R >,
                E : std::convert::From< <#struct_key_ident as toql :: from_row :: FromRow < R >> :: Error>,
                E: std::convert ::From<toql :: sql_builder :: sql_builder_error ::  SqlBuilderError>, */
             //   #(#index_type_bounds)*
  
                {
                    fn index<'a>( mut descendents: &mut toql::query::field_path::Descendents<'a>, field: &str, 
                                rows: &[$row_type], row_offset: usize, index: &mut std::collections::HashMap<u64,Vec<usize>>) 
                        -> std::result::Result<(), $error_type>
                        
                         {
                          
                        use toql::from_row::FromRow;
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        use std::collections::hash_map::DefaultHasher;
                         use toql::sql_builder::select_stream::Select;

                      match descendents.next() {
                            
                                Some(d) => { 
                                    match d.as_str() {
                                        #(#dispatch_index_code),* 
                                        f @ _ => {
                                            return Err(
                                               toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                        }
                                    }
                                },
                                None => {
                                   
                                        let mut  i= row_offset;
                                        for (n, row) in rows.into_iter().enumerate() {
                                            let mut iter = std::iter::repeat(&Select::Query);
                                            let fk = #struct_key_ident ::from_row_with_index(&row, &mut i, &mut iter)?; // SKip Primary key
                                          
                                            let mut s = DefaultHasher::new();
                                            fk.hash(&mut s);
                                           /*  match field {
                                               #(#index_code)*
                                               
                                                f @ _ => {
                                                    return Err(
                                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                                }

                                            }; */
                                            let fk_hash =  s.finish();

                                            index.entry(fk_hash)
                                            .and_modify(|h| h.push(n))
                                            .or_insert(vec![n]);
                                        }
                                
                                        
                                       
                                    }
                                   
                                   
                                
                        }
                        Ok(())  
                    }  
                      
                }
                };
            }

 macro_rules! #macro_name_merge {
                // `()` indicates that the macro takes no argument.
                ($row_type: ty, $error_type: ty) => {
                
                
                impl toql::tree::tree_merge::TreeMerge<$row_type, $error_type> for #struct_ident 
               /*  where Self: toql::from_row::FromRow<R>,
                #struct_key_ident : toql :: from_row :: FromRow < R >,
                E : std::convert::From< <#struct_key_ident as toql :: from_row :: FromRow < R >> :: Error>,
                E: std::convert ::From<toql :: sql_builder :: sql_builder_error ::  SqlBuilderError>,
                E: std::convert ::From<toql :: error ::  ToqlError>,
                #(#merge_type_bounds)* */
  
                {
                    fn merge<'a>(  &mut self, mut descendents: &mut toql::query::field_path::Descendents<'a>, field: &str, 
                                rows: &[$row_type],row_offset: usize, index: &std::collections::HashMap<u64,Vec<usize>>, selection_stream: &toql::sql_builder::select_stream::SelectStream) 
                        -> std::result::Result<(), $error_type>
                        
                         {
                        use toql::key::Keyed;
                        use toql::from_row::FromRow;
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        use std::collections::hash_map::DefaultHasher;
                        use toql::sql_builder::select_stream::Select;

                      match descendents.next() {
                            
                                Some(d) => { 
                                    match d.as_str() {
                                        #(#dispatch_merge_code),* 
                                        f @ _ => {
                                            return Err(
                                               toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                        }
                                    }
                                },
                                None => {

                                        let pk = self.try_get_key()?;
                                        let mut s = DefaultHasher::new();
                                        pk.hash(&mut s);
                                        let h = s.finish();
                                        let default_vec: Vec<usize>= Vec::new();
                                        let row_numbers : &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                                        let  n = row_offset;
                                        
                                        match field {
                                            #(#merge_code)*
                                            
                                            f @ _ => {
                                                return Err(
                                                    toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                            }

                                        };
                                       
                                    }
                                   
                                   
                                
                        }
                        Ok(())  
                    }  
                      
                }
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
