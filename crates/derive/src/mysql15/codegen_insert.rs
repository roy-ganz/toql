/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::Ident;

pub(crate) struct GeneratedMysqlInsert<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,

    insert_columns_code: Vec<TokenStream>,

    insert_values_code: Vec<TokenStream>,
    insert_roles: &'a Option<String>,
    duplicate: bool,
}

impl<'a> GeneratedMysqlInsert<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlInsert {
        GeneratedMysqlInsert {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            insert_columns_code: Vec::new(),

            insert_values_code: Vec::new(),
            insert_roles: &toql.roles.insert,
            duplicate: false,
        }
    }

    pub(crate) fn add_insert_field(&mut self, field: &crate::sane::Field) {
        if field.skip_mut {
            return;
        }

        let rust_field_ident = &field.rust_field_ident;
        let rust_type_ident = &field.rust_type_ident;

        match &field.kind {
            FieldKind::Regular(regular_attrs) => {
                match &regular_attrs.sql_target {
                    SqlTarget::Column(ref sql_column) => self
                        .insert_columns_code
                        .push(quote!(  columns.push(String::from(#sql_column));)),
                    SqlTarget::Expression(_) => {
                        return;
                    }
                };

                self.insert_values_code.push( match field.number_of_options {
                    2 => {
                        // Option<Option<T>> (toql selectable of nullable column)
                        quote!(
                             if  let Some(field) = &entity . #rust_field_ident  {
                                 insert_stmt.push_str("?, ");
                                 params.push( toql::sql_arg::SqlArg::from(field.as_ref()));
                             } else {
                                insert_stmt.push_str("DEFAULT, ");
                             }
                        )
                    }
                    1 if field.preselect => {
                        // Option<T>  selected (nullable column)
                        quote!(
                            insert_stmt.push_str("?, ");
                            params.push( toql::sql_arg::SqlArg::from(entity . #rust_field_ident.as_ref()));
                        )
                    }
                    1 if !field.preselect => {
                        // Option<T>  (toql selectable)
                        quote!(
                            if  let Some(field) = &entity . #rust_field_ident {
                                 insert_stmt.push_str("?, ");
                                  params.push( toql::sql_arg::SqlArg::from(field));
                            } else {
                                 insert_stmt.push_str("DEFAULT, ");
                            }
                        )
                    }
                    _ => {
                        // selected field
                        quote!(
                            insert_stmt.push_str("?, ");
                            params.push(  toql::sql_arg::SqlArg::from(&entity . #rust_field_ident));
                        )
                    }
                });

                // Structs with keys that are insertable may have duplicates
                // Implement marker trait for them
                if regular_attrs.key && !field.skip_mut {
                    self.duplicate = true;
                }
            }
            FieldKind::Join(join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;

                self.insert_columns_code.push(quote!(
                     for other_column in <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns() {
                            #default_self_column_code;
                            let self_column = #columns_map_code;
                        columns.push(self_column.to_owned());
                     }
                ));

                self.insert_values_code.push(
                      match field.number_of_options  {
                                2 => { // Option<Option<T>>
                                        quote!(
                                            if let Some(field) = &entity. #rust_field_ident {
                                                 <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("?, "));
                                                params.extend_from_slice(
                                                   & field.as_ref()
                                                   .map_or_else::<Result<Vec<toql::sql_arg::SqlArg>,toql::error::ToqlError>,_,_>(| |{ Ok(<<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter()
                                                            .map(|_| toql::sql_arg::SqlArg::Null())
                                                            .collect::<Vec<_>>())},
                                                   | some| {Ok(toql::key::Key::params( &<#rust_type_ident as toql::key::Keyed>::try_get_key(&some)?))})?
                                               );
                                            } else {
                                                <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("DEFAULT, "));
                                            }

                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T> 
                                // TODO Option wrapping
                                    quote!(
                                         <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("?, "));
                                         params.extend_from_slice(
                                                   &entity
                                                    . #rust_field_ident .as_ref()
                                                   .map_or_else::<Result<Vec<toql::sql_arg::SqlArg>,toql::error::ToqlError>,_,_>(| |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                                    .map(|_| toql::sql_arg::SqlArg::Null()).collect::<Vec<_>>())},
                                                   | some| { Ok(toql::key::Key::params( &<#rust_type_ident as toql::key::Keyed>::try_get_key(some)?))})?
                                                );
                                           )
                                },

                                1 if !field.preselect => { // Option<T> selectable 
                                    quote!(
                                        if let Some(field) = &entity. #rust_field_ident {
                                            <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("?, "));
                                             params.extend_from_slice(
                                                &toql::key::Key::params(
                                                            &<#rust_type_ident as toql::key::Keyed>::try_get_key(field)?));
                                        } else {
                                              <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("DEFAULT, "));
                                        }
                                    )
                                },
                                _ => { // T
                                    quote!(
                                        <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_|  insert_stmt.push_str("?, "));
                                        params.extend_from_slice(&toql::key::Key::params( &<#rust_type_ident as toql::key::Keyed>::try_get_key(&entity. #rust_field_ident)?));
                                   )
                                }
                            }
                );
                if join_attrs.key && !field.skip_mut {
                    self.duplicate = true;
                }
            }
            FieldKind::Merge(_) => {
                return;
            }
        }
    }
}
impl<'a> quote::ToTokens for GeneratedMysqlInsert<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let insert_values_code = &self.insert_values_code;

        let mods = {
            let insert_statement = format!("INSERT {{}}{{}}INTO {} ({{}}) VALUES ", self.sql_table_name);
            let insert_columns_code = &self.insert_columns_code;

            let role_test = match &self.insert_roles {
                None => quote!(),
            Some(roles) => {
                 quote!(
                    if toql::role_validator::RoleValidator::is_valid(roles, toql::role_parser::RoleParser::parse(#roles)?) {
                        toql::error::ToqlError::SqlBuilderError(toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(#roles))
                    }
                )
                }
            };

            let optional_insert_duplicate_impl = if self.duplicate {
                quote!( impl toql::mutate::InsertDuplicate for #struct_ident {})
            } else {
                quote!()
            };

            quote! {
                #optional_insert_duplicate_impl

                impl toql::mutate::InsertSql for #struct_ident {

                     fn insert_many_sql<Q : std::borrow::Borrow<#struct_ident>>(entities: &[Q], 
                        roles: &std::collections::HashSet<String>,
                        modifier: &str, extra: &str)
                     -> Result<Option< toql::sql::Sql>, toql :: error:: ToqlError>
                     {
                            #role_test

                            if entities.is_empty() {
                                return Ok(None);
                            }

                            let mut params :Vec<toql::sql_arg::SqlArg>= Vec::new();
                            let mut columns :Vec<String>= Vec::new();

                             


                            #(#insert_columns_code)*

                           
                            let mut insert_stmt = format!( #insert_statement, modifier, if modifier.is_empty(){""}else {" "}, columns.join(", "));


                            for bentity in entities {
                                let entity = bentity.borrow();
                                insert_stmt.push('(');
                                #(#insert_values_code)*
                                insert_stmt.pop(); // remove ', '
                                insert_stmt.pop();
                                insert_stmt.push_str("), ");
                            }
                            insert_stmt.pop(); // remove ', '
                            insert_stmt.pop();
                            if !extra.is_empty() {
                                insert_stmt.push(' ');
                                insert_stmt.push_str(extra);
                            };
                            Ok(Some(Sql(insert_stmt, params)))
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
