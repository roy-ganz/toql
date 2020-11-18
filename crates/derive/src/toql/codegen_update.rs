/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::Ident;

pub(crate) struct CodegenUpdate<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    
    sql_table_alias: String,

  

    update_set_code: Vec<TokenStream>,
  
    struct_upd_roles: &'a HashSet<String>,
    

      dispatch_update_code: Vec<TokenStream>,
}

impl<'a> CodegenUpdate<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenUpdate {
        CodegenUpdate {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            sql_table_alias : toql.sql_table_alias.to_owned(),
           
            update_set_code: Vec::new(),
           
            struct_upd_roles: &toql.upd_roles,

            dispatch_update_code: Vec::new()
        }
    }

    pub(crate) fn add_tree_update(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_type_ident = &field.rust_type_ident;
         let toql_field_name= &field.toql_field_name;
         let sql_table_alias = &self.sql_table_alias;

        let unwrap = match field.number_of_options {
                    1 => quote!(.as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                    0 => quote!(),
                    _ => quote!(.as_ref().unwrap().as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?),
                };

                let refer = match field.number_of_options {
                    0 => quote!(&),
                    _ => quote!(),
                };

        // Handle key predicate and parameters
        match &field.kind {
            FieldKind::Regular(regular_attrs) => {
                // SQL code cannot be updated, skip field
                if let SqlTarget::Expression(_) = regular_attrs.sql_target {
                    return;
                };

                if !field.skip_mut{

                    let role_assert = if field.load_roles.is_empty() {
                        quote!()
                    } else {
                        let roles = &field.load_roles;
                        quote!(
                            if toql::query::assert_roles(roles, &[ #(String::from(#roles)),* ].iter().cloned().collect()).is_ok()
                        )
                    };


                    let column_set =  if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                            quote!(
                                    expr.push_alias(#sql_table_alias);
                                            expr.push_literal(".");
                                            expr.push_literal(#sql_column);
                                            expr.push_literal(" = ");
                                            expr.push_arg(toql::sql_arg::SqlArg::from(self . #rust_field_ident.as_ref().unwrap()));
                                            expr.push_literal(", ");
                                    )
                        } else {
                          quote!()
                        };

                    // Selectable fields
                    // Option<T>, <Option<Option<T>>
                    if field.number_of_options > 0 && !field.preselect {
                        /* let unwrap_null = if 2 == field.number_of_options {
                            quote!(.as_ref().map_or(String::from("NULL"), |x| x.to_string()))
                        } else {
                            quote!()
                        }; */


                       

                        // update statement
                        // Doesn't update primary key
                        if !regular_attrs.key {
                            self.update_set_code.push(quote!(
                                #role_assert {
                                    if  self. #rust_field_ident .is_some() 
                                        && (fields.contains("*") || fields.contains( #toql_field_name)){
                                            #column_set
                                    }
                                }
                            ));
                        }
                        
                    }
                    // Not selectable fields
                    // T, Option<T> (nullable column)
                    else {
                    
                        //update statement
                        if !regular_attrs.key {
                            self.update_set_code.push(quote!(
                            #role_assert {
                                 if fields.contains("*") || fields.contains( #toql_field_name) {
                                    #column_set
                                 }
                            }
                            ));
                        }

                    }
                   
                }
            }
            FieldKind::Join(_join_attrs) => {
               
                self.dispatch_update_code.push(
                   quote!(
                      #toql_field_name => { 
                            <#rust_type_ident as toql::tree::tree_update::TreeUpdate>::
                            update(#refer  self. #rust_field_ident # unwrap ,&mut descendents, fields, roles, exprs)?
                        }
                )
               );
            }
            FieldKind::Merge(_) => {
                let rust_base_type_ident = &field.rust_base_type_ident;
                self.dispatch_update_code.push(
                    match field.number_of_options {
                        1 => {quote!(
                                #toql_field_name => { 
                                    if let Some (fs) = self. #rust_field_ident .as_ref(){
                                        for f in fs {
                                            <#rust_base_type_ident as toql::tree::tree_update::TreeUpdate>::update(f, &mut descendents, fields, roles, exprs)?
                                        }
                                    }
                                }
                        )}
                        _ => {
                            quote!(
                                #toql_field_name => { 
                                    for f in self. #rust_field_ident .as_ref(){
                                        <#rust_base_type_ident as toql::tree::tree_update::TreeUpdate>::update(f, &mut descendents,fields,  roles, exprs)?
                                    }
                                }
                            )
                        }

                    }
                   
               );
                  
            }
        };

      

      
    }
}
impl<'a> quote::ToTokens for CodegenUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let update_set_code = &self.update_set_code;
       

      

      

        // Generate modules if there are keys available
        let mods = {
           
            let sql_table_name = &self.sql_table_name;
            let sql_table_alias = &self.sql_table_alias;

            let upd_role_test = if self.struct_upd_roles.is_empty() {
                quote!()
            } else {
                let roles = &self.struct_upd_roles;
                quote!(
                        toql::query::assert_roles(roles, &[ #(String::from(#roles)),* ].iter().cloned().collect())
                        .map_err(|e|toql::error::ToqlError::SqlBuilderError(toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(e)))?;

                )
            };
          
            let dispatch_update_code = &self.dispatch_update_code;

            quote! {

                impl toql::tree::tree_update::TreeUpdate for #struct_ident {

                    #[allow(unused_mut, unused_variables)]
                    fn update<'a>(&self, mut descendents: &mut  toql::query::field_path::Descendents<'a>, 
                    fields: &std::collections::HashSet<String>, roles: &std::collections::HashSet<String>, 
                    exprs : &mut Vec<toql::sql_expr::SqlExpr>) -> Result<(), toql::error::ToqlError>{

                                match descendents.next() {
                                                            
                                    Some(d) => { 
                                        match d.as_str() {
                                            #(#dispatch_update_code),* 
                                            f @ _ => {
                                                return Err(
                                                toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                            }
                                        }
                                    },
                                    None => {
                                        #upd_role_test

                                        let mut expr = toql::sql_expr::SqlExpr::new();
                                        expr.push_literal("UPDATE ");
                                        expr.push_literal(#sql_table_name);
                                         expr.push_literal(" ");
                                        expr.push_alias(#sql_table_alias);
                                        expr.push_literal(" SET ");
                                        let tokens = expr.tokens().len();
                                        #(#update_set_code)*
                                        expr.pop_literals(2);
                                        if expr.tokens().len() > tokens {
                                            expr.push_literal(" WHERE ");
                                            let key = <Self as toql::key::Keyed>::try_get_key(&self)?;
                                            let resolver = toql::sql_expr::resolver::Resolver::new().with_self_alias(#sql_table_alias);
                                            expr.extend( resolver.resolve(&toql::key::predicate_expr(key))?); 
                                            exprs.push(expr);
                                        }
                                    }
                                };
                              
                                Ok(())
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
