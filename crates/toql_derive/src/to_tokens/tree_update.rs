use crate::parsed::{
    field::{
        field_kind::FieldKind,
        join_field::JoinSelection,
        merge_field::MergeSelection,
        regular_field::{RegularSelection, SqlTarget},
    },
    parsed_struct::ParsedStruct,
};
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut update_set_code = Vec::new();
    let mut dispatch_update_code = Vec::new();

    let struct_name = parsed_struct.struct_name.to_string();

    for field in &parsed_struct.fields {
        if field.skip_mut {
            continue;
        }
        let field_name_ident = &field.field_name;
        let field_base_type_path = &field.field_base_type;
        let toql_query_name = &field.toql_query_name;

        let role_assert = match &field.roles.update {
            Some(role_expr_string) => {
                quote!(
                   if !role_valid {
                         return Err(toql::error::ToqlError::SqlBuilderError(
                             toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(role_expr .to_string(), format!("field `{}` on mapper `{}`", #toql_query_name,  #struct_name ))));
                   }
                )
            }
            None => quote!(),
        };

        let role_valid_code = match &field.roles.update {
            Some(role_expr_string) => {
                quote!(
                    let role_expr = toql::role_expr_macro::role_expr!(#role_expr_string);
                   let role_valid = toql::role_validator::RoleValidator::is_valid(roles, &role_expr);

                )
            }
            None => quote!(),
        };
        let role_predicate = if field.roles.update.is_some() {
            quote!(role_valid &&)
        } else {
            quote!()
        };

        // Handle key predicate and parameters
        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(regular_kind) => {
                // SQL code cannot be updated, skip field
                if let SqlTarget::Expression(_) = regular_kind.sql_target {
                    continue;
                };
                if regular_kind.key {
                    continue;
                }

                let value = if regular_kind.selection != RegularSelection::Preselect {
                    quote!( self. #field_name_ident .as_ref()
                        .map(|f|toql::sql_arg::SqlArg::from(f.to_owned()))
                        .unwrap_or(toql::sql_arg::SqlArg::Null)
                    )
                } else {
                    quote!( toql::sql_arg::SqlArg::from(&self . #field_name_ident))
                };

                let column_set = if let SqlTarget::Column(ref sql_column) = &regular_kind.sql_target
                {
                    quote!(
                            expr.push_literal(#sql_column);
                            expr.push_literal(" = ");
                            expr.push_arg( #value);
                            expr.push_literal(", ");
                    )
                } else {
                    quote!()
                };

                let opt_field_predicate = if regular_kind.selection == RegularSelection::Select
                    || regular_kind.selection == RegularSelection::SelectNullable
                {
                    // Selectable fields: Option<T>, <Option<Option<T>>
                    quote!(self. #field_name_ident .is_some() && )
                }
                // Not selectable fields: T, Option<T> (nullable column)
                else {
                    quote!()
                };
                update_set_code.push(quote!(
                            #role_valid_code
                            if #opt_field_predicate ((#role_predicate path_selected) || fields.contains( #toql_query_name)) {
                                #role_assert
                                #column_set
                            }
                        ));
            }
            FieldKind::Join(join_attrs) => {
                let refer = if join_attrs.selection == JoinSelection::PreselectInner {
                    quote!(&)
                } else {
                    quote!()
                };

                dispatch_update_code.push(
                    match join_attrs.selection {
                        JoinSelection::PreselectInner => { //T
                            quote!(
                                #toql_query_name => {
                                        toql::tree::tree_update::TreeUpdate::update(#refer  self. #field_name_ident , descendents, fields, roles, exprs)?
                                    }
                            )

                        },
                        JoinSelection::PreselectLeft | JoinSelection::SelectInner => { // Option<T>
                                quote!(
                                #toql_query_name => {
                                        if let Some(f) = self. #field_name_ident .as_ref() {
                                        toql::tree::tree_update::TreeUpdate::update( f , descendents, fields, roles, exprs)?
                                        }
                                    }
                            )
                        }
                        JoinSelection::SelectLeft => { // Option<Option<T>>
                                quote!(
                                #toql_query_name => {
                                        if let Some(f1) = self. #field_name_ident .as_ref() {
                                            if let Some(f2) = f1 {
                                                    toql::tree::tree_update::TreeUpdate::update(f2 , descendents, fields, roles, exprs)?
                                                }
                                        }
                                    }
                            )
                        }

                    }
                );

                // Key fields cannot be updated
                if join_attrs.key {
                    continue;
                }

                let mut inverse_column_translation: Vec<TokenStream> = Vec::new();

                let args_code = match join_attrs.selection {
                        JoinSelection::SelectLeft =>  quote!(let args = if let Some(entity) = self. #field_name_ident.as_ref() .unwrap() {
                                    toql::key::Key::params(&toql::keyed::Keyed::key(& entity))
                                } else {
                                    inverse_columns.iter().map(|c| toql::sql_arg::SqlArg::Null).collect::<Vec<_>>()
                                };),
                        JoinSelection::SelectInner | JoinSelection::PreselectLeft =>  quote!(let args =  toql::key::Key::params(&toql::keyed::Keyed::key( &self. #field_name_ident .as_ref() .unwrap()));),
                        JoinSelection::PreselectInner =>   quote!(let args =  toql::key::Key::params(&toql::keyed::Keyed::key(&self. #field_name_ident));)
                    };

                let opt_field_predicate = if join_attrs.selection == JoinSelection::SelectLeft
                    || join_attrs.selection == JoinSelection::SelectInner
                {
                    // Selectable fields: Option<T>, Option<Option<T>>
                    quote!(self. #field_name_ident .is_some() && )
                } else {
                    // Not selectable fields: T, Option<T> (nullable column)
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
                       let default_inverse_columns= <<#field_base_type_path as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns();
                       let inverse_columns = <<#field_base_type_path as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().enumerate().map(|(i, c)| {
                            let inverse_column = match c.as_str() {
                                    #(#inverse_column_translation)*
                                _ => {
                                        default_inverse_columns.get(i).unwrap().to_owned()
                                    }
                            };
                            inverse_column
                        }).collect::<Vec<String>>();
                    )
                } else {
                    // default column naming code
                    let column_format = format!("{}_{{}}", field_name_ident);
                    quote!(
                          let inverse_columns =
                    <<#field_base_type_path as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
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
                    update_set_code.push(
                        quote!(
                            #role_valid_code
                            if #opt_field_predicate ((#role_predicate path_selected) || fields.contains( #toql_query_name)) {
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
            FieldKind::Merge(merge_attrs) => {
                dispatch_update_code.push(
                        match merge_attrs.selection {
                            MergeSelection::Preselect => {
                                // Vec<T>
                                quote!(
                                    #toql_query_name => {
                                        for f in &self. #field_name_ident{
                                            toql::tree::tree_update::TreeUpdate::update(f,  descendents.clone() ,fields,  roles, exprs)?
                                        }
                                    }
                                )
                            }
                            MergeSelection::Select => {
                                // Option<Vec<T>>
                                quote!(
                                    #toql_query_name => {
                                        if let Some (fs) = self. #field_name_ident .as_ref(){
                                            for f in fs {
                                               toql::tree::tree_update::TreeUpdate::update(f,  descendents.clone(), fields, roles, exprs)?
                                            }
                                        }
                                    }
                            )}
                        }
                );
            }
        };
    }

    // Generate Stream
    let struct_name_ident = &parsed_struct.struct_name;
    let struct_name = parsed_struct.struct_name.to_string();
    let mods = {
        let sql_table_name = &parsed_struct.table;

        let struct_upd_role_assert = match &parsed_struct.roles.update {
            None => quote!(),
            Some(role_expr_string) => {
                quote!(
                    let role_expr = toql::role_expr_macro::role_expr!(#role_expr_string);
                    if !toql::role_validator::RoleValidator::is_valid(roles, &role_expr)  {
                        return Err(toql::error::ToqlError::SqlBuilderError(
                            toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(role_expr.to_string(), format!("mapper `{}`", #struct_name))));
                    }
                )
            }
        };

        quote! {
            impl toql::tree::tree_update::TreeUpdate for #struct_name_ident {

                #[allow(unused_mut, unused_variables, unused_parens)]
                fn update<'a, I>(&self, mut descendents: I,
                fields: &std::collections::HashSet<String>, roles: &std::collections::HashSet<String>,
                exprs : &mut Vec<toql::sql_expr::SqlExpr>) -> std::result::Result<(), toql::error::ToqlError>
                where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                            let key = <Self as toql::keyed::Keyed>::key(&self);
                            if !toql::sql_arg::valid_key(&toql::key::Key::params(&key)) {
                                return Ok(())
                            }
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
                                        expr.extend( resolver.alias_to_literals(&toql::key::Key::unaliased_predicate_expr(&key))?);
                                        exprs.push(expr);
                                    }

                                }
                            };

                            Ok(())
                }
            }

            impl toql::tree::tree_update::TreeUpdate for &mut #struct_name_ident {
                #[allow(unused_mut)]
                fn update<'a, I>(&self, mut descendents:  I,
                fields: &std::collections::HashSet<String>, roles: &std::collections::HashSet<String>,
                exprs : &mut Vec<toql::sql_expr::SqlExpr>) -> std::result::Result<(), toql::error::ToqlError>
                where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                    <#struct_name_ident as toql::tree::tree_update::TreeUpdate>::update(self, descendents, fields, roles, exprs)
                }
            }
        }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
