/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use syn::Ident;

pub(crate) struct CodegenUpdate<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    update_set_code: Vec<TokenStream>,
    struct_upd_roles: &'a Option<String>,
    dispatch_update_code: Vec<TokenStream>,
}

impl<'a> CodegenUpdate<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenUpdate {
        CodegenUpdate {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            update_set_code: Vec::new(),
            struct_upd_roles: &toql.roles.update,
            dispatch_update_code: Vec::new(),
        }
    }

    pub(crate) fn add_tree_update(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_type_ident = &field.rust_type_ident;
        let toql_field_name = &field.toql_field_name;
       
/* 
        let unwrap = match field.number_of_options {
            1 => {
                quote!(.as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
            0 => quote!(),
            _ => {
                quote!(.as_ref().unwrap().as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
        }; */

        let refer = match field.number_of_options {
            0 => quote!(&),
            _ => quote!(),
        };

        let role_assert = match &field.roles.update {
            None => quote!(),
            Some(role_expr_string) => {
                quote!(
                   if !toql::role_validator::RoleValidator::is_valid(roles,
                   &toql::role_expr_macro::role_expr!(#role_expr_string)) {
                         return Err(toql::error::ToqlError::SqlBuilderError(toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(#role_expr_string .to_string())));
                   }
                )
            }
        };
        let role_predicate = match &field.roles.update {
            None => quote!(),
            Some(role_expr_string) => {
                quote!(
                   (toql::role_validator::RoleValidator::is_valid(roles,
                   &toql::role_expr_macro::role_expr!(#role_expr_string)) &&
                   fields.contains("*")) &&
                )
            }
        };

        // Handle key predicate and parameters
        match &field.kind {
            FieldKind::Regular(regular_attrs) => {
                // SQL code cannot be updated, skip field
                if let SqlTarget::Expression(_) = regular_attrs.sql_target {
                    return;
                };
                if field.skip_mut || regular_attrs.key {
                    return;
                }

                let value = if field.number_of_options > 0 {
                    quote!( self. #rust_field_ident .as_ref().map(|f|toql::sql_arg::SqlArg::from(f)).unwrap_or(toql::sql_arg::SqlArg::Null))
                } else {
                    quote!( toql::sql_arg::SqlArg::from(&self . #rust_field_ident))
                };

                let column_set =
                    if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                        quote!(
                        // expr.push_alias(#sql_table_alias);
                            //    expr.push_literal(".");
                                expr.push_literal(#sql_column);
                                expr.push_literal(" = ");
                                expr.push_arg( #value);
                                expr.push_literal(", ");
                        )
                    } else {
                        quote!()
                    };

                // Selectable fields
                // Option<T>, <Option<Option<T>>
                let opt_field_predicate = if field.number_of_options > 0 && !field.preselect {
                    quote!(self. #rust_field_ident .is_some() && )
                }
                // Not selectable fields
                // T, Option<T> (nullable column)
                else {
                    quote!()
                };
                self.update_set_code.push(quote!(
                            if #opt_field_predicate #role_predicate (path_selected || fields.contains( #toql_field_name)) {
                                #role_assert
                                #column_set
                            }
                        ));
            }
            FieldKind::Join(join_attrs) => {
                if !(field.skip_mut || join_attrs.key) {
                    self.dispatch_update_code.push(
                        match field.number_of_options {
                            0 => {
                                quote!(
                                    #toql_field_name => {
                                            #role_assert

                                            <#rust_type_ident as toql::tree::tree_update::TreeUpdate>::
                                            update(#refer  self. #rust_field_ident , descendents, fields, roles, exprs)?
                                        }
                                )

                            },
                           
                            1 => {
                                 quote!(
                                    #toql_field_name => {
                                            #role_assert
                                            if let Some(f) = self. #rust_field_ident .as_ref() {
                                            <#rust_type_ident as toql::tree::tree_update::TreeUpdate>::
                                            update( f , descendents, fields, roles, exprs)?
                                            }
                                        }
                                )
                            }
                            _ => { // 2
                                 quote!(
                                    #toql_field_name => {
                                            #role_assert
                                            if let Some(f1) = self. #rust_field_ident .as_ref() {
                                                if let Some(f2) = f1 {
                                                    <#rust_type_ident as toql::tree::tree_update::TreeUpdate>::
                                                    update(f2 , descendents, fields, roles, exprs)?
                                                    }
                                            }
                                        }
                                )
                            }

                        }
                    );

                    let mut inverse_column_translation: Vec<TokenStream> = Vec::new();

                    let args_code = match field.number_of_options {
                        2 =>  quote!(let args = if let Some(entity) = self. #rust_field_ident.as_ref() .unwrap() {
                                    toql::key::Key::params(&toql::keyed::Keyed::key(& entity))
                                } else {
                                    inverse_columns.iter().map(|c| toql::sql_arg::SqlArg::Null).collect::<Vec<_>>()
                                };),
                        1 =>  quote!(let args =  toql::key::Key::params(&toql::keyed::Keyed::key( &self. #rust_field_ident .as_ref() .unwrap()));),
                        _ =>   quote!(let args =  toql::key::Key::params(&toql::keyed::Keyed::key(&self. #rust_field_ident));)
                    };

                    let opt_field_predicate = if field.number_of_options > 0 && !field.preselect {
                        quote!(self. #rust_field_ident .is_some() && )
                    }
                    // Not selectable fields
                    // T, Option<T> (nullable column)
                    else {
                        quote!()
                    };

                    for m in &join_attrs.columns {
                        let untranslated_column = &m.this;
                        let other_column = &m.other;

                        inverse_column_translation
                            .push(quote!( #other_column => String::from(#untranslated_column),));
                    }

                    let columns_code = if !join_attrs.columns.is_empty() { 
                        // column translation code
                        quote!(
                       let default_inverse_columns= <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns();
                       let inverse_columns = <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().enumerate().map(|(i, c)| {
                            let inverse_column = match c.as_str() {
                                    #(#inverse_column_translation)*
                                _ => {
                                        default_inverse_columns.get(i).unwrap().to_owned()
                                    }
                            };
                            inverse_column
                        }).collect::<Vec<String>>();
                    ) } else {
                        // default column naming code
                        let column_format = format!("{}_{{}}", rust_field_ident);
                        quote!(
                              let inverse_columns =
                        <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                format!(#column_format, c)
                            })
                            .collect::<Vec<String>>(); 
                        )
                    };

                    // Add if columns should not be skipped
                    if !join_attrs.partial_table {
                        self.update_set_code.push(
                        quote!(
                            if #opt_field_predicate #role_predicate (path_selected || fields.contains( #toql_field_name)) {
                                #role_assert
                                #columns_code
                                #args_code
                                for (c, a) in inverse_columns.iter().zip(args) {
                                    expr.push_literal(c);
                                    expr.push_literal(" = ");
                                    expr.push_arg(a);
                                    expr.push_literal(", ");
                                }
                            }
                        )
                        );
                    }
                }
            }
            FieldKind::Merge(_) => {
                if !field.skip_mut {
                    let rust_base_type_ident = &field.rust_base_type_ident;
                    self.dispatch_update_code.push(
                        match field.number_of_options {
                            0 => {
                                quote!(
                                    #toql_field_name => {
                                        #role_assert
                                        for f in &self. #rust_field_ident{
                                            <#rust_base_type_ident as toql::tree::tree_update::TreeUpdate>::update(f,  descendents.clone() ,fields,  roles, exprs)?
                                        }
                                    }
                                )
                            }
                            _ => {quote!(
                                    #toql_field_name => {
                                        #role_assert
                                        if let Some (fs) = self. #rust_field_ident .as_ref(){
                                            for f in fs {
                                                <#rust_base_type_ident as toql::tree::tree_update::TreeUpdate>::update(f,  descendents.clone(), fields, roles, exprs)?
                                            }
                                        }
                                    }
                            )}
                        }
                );
                }
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

            let struct_upd_role_assert = match self.struct_upd_roles {
                None => quote!(),
                Some(role_expr_string) => {
                    quote!(
                        if !toql::role_validator::RoleValidator::is_valid(roles, &&toql::role_expr_macro::role_expr!(#role_expr_string))  {
                            return Err(toql::error::ToqlError::SqlBuilderError(toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(#role_expr_string .to_string())));
                        }
                    )
                }
            };

            let dispatch_update_code = &self.dispatch_update_code;

            quote! {

                impl toql::tree::tree_update::TreeUpdate for #struct_ident {

                    #[allow(unused_mut, unused_variables, unused_parens)]
                    fn update<'a, I>(&self, mut descendents: I,
                    fields: &std::collections::HashSet<String>, roles: &std::collections::HashSet<String>,
                    exprs : &mut Vec<toql::sql_expr::SqlExpr>) -> std::result::Result<(), toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                    {

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
                                        let key = <Self as toql::keyed::Keyed>::key(&self);
                                        if !toql::sql_arg::valid_key(&toql::key::Key::params(&key)) {
                                            return Ok(())
                                        }
                                        
                                        let path_selected = fields.contains("*");
                                        #struct_upd_role_assert

                                        let mut expr = toql::sql_expr::SqlExpr::new();
                                        expr.push_literal("UPDATE ");
                                        expr.push_literal(#sql_table_name);
                                        expr.push_literal(" SET ");
                                        let tokens = expr.tokens().len();
                                        #(#update_set_code)*

                                       expr.pop(); // remove ', '
                                        if expr.tokens().len() > tokens {
                                            expr.push_literal(" WHERE ");
                                            let key = <Self as toql::keyed::Keyed>::key(&self);
                                            //let resolver = toql::sql_expr::resolver::Resolver::new().with_self_alias(#sql_table_alias);
                                            // Qualifierd column name
                                            let resolver = toql::sql_expr::resolver::Resolver::new().with_self_alias(#sql_table_name);
                                            expr.extend( resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key, false))?);
                                            exprs.push(expr);
                                        }

                                    }
                                };

                                Ok(())
                    }
                }

                  impl toql::tree::tree_update::TreeUpdate for &mut #struct_ident {

                    #[allow(unused_mut)]
                    fn update<'a, I>(&self, mut descendents:  I,
                    fields: &std::collections::HashSet<String>, roles: &std::collections::HashSet<String>,
                    exprs : &mut Vec<toql::sql_expr::SqlExpr>) -> std::result::Result<(), toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                    {
                        <#struct_ident as toql::tree::tree_update::TreeUpdate>::update(self, descendents, fields, roles, exprs)
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
