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
    let mut insert_columns_code = Vec::new(); // For fields
    let mut insert_values_code = Vec::new();

    let mut dispatch_columns_code = Vec::new(); // for joins
    let mut dispatch_values_code = Vec::new();

    for field in &parsed_struct.fields {
        if field.skip_mut {
            continue;
        }
        let field_name_ident = &field.field_name;
        let field_base_type = &field.field_base_type;
        let field_query_name = &field.toql_query_name;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(regular_kind) => {
                if regular_kind.key && parsed_struct.auto_key {
                    continue;
                }
                match regular_kind.sql_target {
                    SqlTarget::Column(ref sql_column) => insert_columns_code.push(quote!(
                                e.push_literal(#sql_column);
                                e.push_literal(", ");
                    )),
                    SqlTarget::Expression(_) => {
                        continue;
                    }
                }
                insert_values_code.push( match regular_kind.selection {
                    RegularSelection::SelectNullable => {
                        // Option<Option<T>> (toql selectable of nullable column)
                        quote!(
                             if  let Some(field) = &self . #field_name_ident  {
                                 values.push_arg(toql::sql_arg::SqlArg::from(field.as_ref()));
                                 values.push_literal(", ");
                             } else {
                                values.push_literal("DEFAULT, ");
                             }
                        )
                    }
                    RegularSelection::PreselectNullable=> {
                        // Option<T>  selected (nullable column)
                        quote!(
                              values.push_arg( toql::sql_arg::SqlArg::from(self . #field_name_ident.as_ref()));
                              values.push_literal(", ");
                        )
                    }
                    RegularSelection::Select => {
                        // Option<T>  (toql selectable)
                        quote!(
                            if  let Some(field) = &self . #field_name_ident {
                                 values.push_arg( toql::sql_arg::SqlArg::from(field));
                                   values.push_literal(", ");
                            } else {
                                  values.push_literal("DEFAULT, ");
                            }
                        )
                    }
                    RegularSelection::Preselect => {
                        // selected field
                        quote!(
                            values.push_arg(toql::sql_arg::SqlArg::from(&self . #field_name_ident));
                            values.push_literal(", ");
                        )
                    }
                });
            }

            FieldKind::Join(join_kind) => {
                if join_kind.key && parsed_struct.auto_key {
                    continue;
                }

                dispatch_columns_code.push(quote!(
                        #field_query_name => {
                            return Ok(<#field_base_type as toql::tree::tree_insert::TreeInsert>::columns(descendents)?);
                        }
                ));

                dispatch_values_code.push(
                   match join_kind.selection  {
                                JoinSelection::SelectLeft => {
                                    // Option<Option<T>
                                    quote!(
                                    #field_query_name => {
                                         if let Some(f) = self. #field_name_ident .as_ref() {
                                              if let Some(f) = f .as_ref() {
                                                toql::tree::tree_insert::TreeInsert::values(f, descendents, roles, should_insert, values)?
                                            }
                                         }
                                    }
                                        ) },
                                JoinSelection::SelectInner | JoinSelection::PreselectLeft => {
                                    // Option<T>
                                    quote!(
                                    #field_query_name => {
                                        if let Some(f) = self. #field_name_ident .as_ref() {
                                            toql::tree::tree_insert::TreeInsert::values(f, descendents, roles, should_insert, values)?
                                        }
                                    }
                                        ) },
                                 JoinSelection::PreselectInner  => {
                                     // T
                                    quote!(
                                        #field_query_name => {
                                           toql::tree::tree_insert::TreeInsert::
                                            values(& self. #field_name_ident, descendents, roles, should_insert, values)?
                                        }
                                    )}
                   }
               );
                let columns_map_code = &join_kind.columns_map_code;
                let default_self_column_code = &join_kind.default_self_column_code;

                // Add if columns should not be skipped

                if !join_kind.partial_table {
                    insert_columns_code.push(quote!(
                        for other_column in <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns() {
                                #default_self_column_code;
                                let self_column = #columns_map_code;
                                e.push_literal(self_column);
                                e.push_literal(", ");
                        }
                    ));

                    insert_values_code.push(
                        match join_kind.selection  {
                                    JoinSelection::SelectLeft => { // Option<Option<T>>
                                            quote!(
                                                if let Some(field) = &self. #field_name_ident {
                                                    if let Some(f) = field {
                                                        toql :: key :: Key :: params(&  toql :: keyed :: Keyed  :: key(f))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                                    } else {
                                                        <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                        .iter().for_each(|_| {
                                                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                                                values.push_literal(", ");});

                                                    }
                                                } else {
                                                    <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_| { values.push_literal("DEFAULT, ");});
                                                }

                                            )
                                        },
                                    JoinSelection::PreselectLeft => { // #[toql(preselect)] Option<T> 
                                    // TODO Option wrapping
                                        quote!(
                                            if let Some(f) =  &self. #field_name_ident {
                                                        toql :: key :: Key :: params(& toql :: keyed :: Keyed  :: key(f))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                                    } else {
                                                        <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                        .iter().for_each(|_| {
                                                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                                                values.push_literal(", ");
                                                                });
                                                    }
                                            )
                                    },

                                    JoinSelection::SelectInner => { // Option<T> selectable 
                                        quote!(
                                            if let Some(field) = &self. #field_name_ident {
                                                        toql :: key :: Key :: params(& toql :: keyed :: Keyed :: key(field))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                            } else {
                                                <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                    .iter().for_each(|_|  {values.push_literal("DEFAULT, ");});
                                            }
                                        )
                                    },
                                    JoinSelection::PreselectInner => { // T
                                        quote!(
                                            &toql::key::Key::params( &toql::keyed::Keyed::key(&self. #field_name_ident))
                                        .into_iter() .for_each(|a| {values.push_arg(a); values.push_literal(", " );});
                                    )
                                    }
                        }
                    );
                }
            }
            FieldKind::Merge(merge_kind) => {
                dispatch_columns_code.push(
                   quote!(
                        #field_query_name => {
                             return Ok(<#field_base_type as toql::tree::tree_insert::TreeInsert>::columns( descendents)?);
                        }
                )
               );
                dispatch_values_code.push(
                    match merge_kind.selection {
                        MergeSelection::Preselect => {
                            // Vec<T>
                            quote!(
                                #field_query_name => {
                                    for f in &self. #field_name_ident{
                                        toql::tree::tree_insert::TreeInsert::values(f, descendents.clone(), roles, should_insert, values)?
                                    }
                                }
                            )
                        }
                         MergeSelection::Select => {
                             // Option<Vec<T>>
                             quote!(
                                #field_query_name => {
                                    if let Some (fs) = self. #field_name_ident .as_ref(){
                                        for f in fs {
                                            toql::tree::tree_insert::TreeInsert::values(f,  descendents.clone(), roles, should_insert, values)?
                                        }
                                    }
                                }
                        )
                        },
                    }
               );
            }
        };
    }
    let struct_name_ident = &parsed_struct.struct_name;
    let struct_name = parsed_struct.struct_name.to_string();
    let role_assert = if let Some(role_expr_string) = &parsed_struct.roles.insert {
        quote!(
            let role_expr = toql::role_expr_macro::role_expr!(#role_expr_string);
            if !toql::role_validator::RoleValidator::is_valid(roles, &role_expr)  {
                return Err( toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(role_expr.to_string(), format!("mapper `{}`", #struct_name) ).into())
            }
        )
    } else {
        quote!()
    };

    let mods = quote! {
            impl toql::tree::tree_insert::TreeInsert for #struct_name_ident {

                #[allow(unused_mut)]
                fn columns<'a, I>(  mut descendents: I)
                        -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                        {

                    let mut e = toql::sql_expr::SqlExpr::new();
                     match descendents.next() {
                           Some(d) => match d.as_str() {
                               #(#dispatch_columns_code),*
                               f @ _ => {
                                    return Err(
                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                }
                           },
                           None => {
                               e.push_literal("(");
                               #(#insert_columns_code)*
                               e.pop_literals(2);
                               e.push_literal(")");
                           }
                    }
                    Ok(e)
                }
                #[allow(unused_mut, unused_variables)]
                fn values<'a,'b, I, J>(&self,
                                    mut descendents: I,
                                    roles: &std::collections::HashSet<String>,
                                     mut should_insert:  &mut J,
                                     values:  &mut toql::sql_expr::SqlExpr
                            ) -> std::result::Result<(),  toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
                             J: Iterator<Item =&'b bool >
                            {

                                match descendents.next() {
                                    Some(d) => match d.as_str() {
                                        #(#dispatch_values_code),*
                                        f @ _ => {
                                                return Err(
                                                    toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                            }
                                    },
                                    None => {
                                          if !*should_insert.next().unwrap_or(&false) {
                                                return Ok(())
                                            }

                                        #role_assert

                                        values.push_literal("(");
                                        #(#insert_values_code)*
                                        values.pop_literals(2);
                                        values.push_literal("), ");
                                    }
                                }
                                Ok(())
                            }
            }

              impl toql::tree::tree_insert::TreeInsert for &#struct_name_ident {

                #[allow(unused_mut)]
                fn columns<'a, I>(  mut descendents: I)
                        -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> {
                            <#struct_name_ident as toql::tree::tree_insert::TreeInsert>::columns(descendents)
                        }
                #[allow(unused_mut)]
                 fn values<'a,'b, I, J>(&self,
                                    mut descendents: I,
                                    roles: &std::collections::HashSet<String>,
                                     mut should_insert:  &mut J,
                                     values:  &mut toql::sql_expr::SqlExpr
                            ) -> std::result::Result<(),  toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
                             J: Iterator<Item =&'b bool >
                            {
                                <#struct_name_ident as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, should_insert, values)
                            }
              }
              impl toql::tree::tree_insert::TreeInsert for &mut #struct_name_ident {

                #[allow(unused_mut)]
                fn columns<'a, I>(  mut descendents: I)
                        -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> {
                            <#struct_name_ident as toql::tree::tree_insert::TreeInsert>::columns(descendents)
                        }
                #[allow(unused_mut)]
                 fn values<'a,'b, I, J>(&self,
                                    mut descendents: I ,
                                    roles: &std::collections::HashSet<String>,
                                     mut should_insert:  &mut J,
                                     values:  &mut toql::sql_expr::SqlExpr
                            ) -> std::result::Result<(),  toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
                             J: Iterator<Item =&'b bool >
                            {
                                <#struct_name_ident as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, should_insert, values)
                            }
              }


    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
