/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use syn::Ident;
use std::collections::HashSet;

pub(crate) struct GeneratedMysqlInsert<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,

    insert_columns_code: Vec<TokenStream>,

    insert_values_code: Vec<TokenStream>,
    insdel_roles: &'a HashSet<String>,
    duplicate: bool

}

impl<'a> GeneratedMysqlInsert<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlInsert {
        GeneratedMysqlInsert {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            insert_columns_code: Vec::new(),

            insert_values_code: Vec::new(),
            insdel_roles : &toql.insdel_roles,
            duplicate: false
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
                                 params.push( field.as_ref()
                                            .map_or(String::from("NULL"), |x| x.to_string().to_owned()));
                             } else {
                                insert_stmt.push_str("DEFAULT,");
                             }
                        )
                    }
                    1 if field.preselect => {
                        // Option<T>  selected (nullable column)
                        quote!(
                            insert_stmt.push_str("?, ");
                            params.push( entity . #rust_field_ident
                            .as_ref()
                            .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                            );
                        )
                    }
                    1 if !field.preselect => {
                        // Option<T>  (toql selectable)
                        quote!(
                            if  let Some(field) = &entity . #rust_field_ident {
                                 insert_stmt.push_str("?, ");
                                  params.push( field.to_string().to_owned());
                            } else {
                                 insert_stmt.push_str("DEFAULT, ");
                            }
                        )
                    }
                    _ => {
                        // selected field
                        quote!(
                            insert_stmt.push_str("?, ");
                            params.push( entity . #rust_field_ident .to_string());
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
                     for other_column in <#rust_type_ident as toql::key::Key>::columns() {
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
                                                 <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("?, "));
                                                params.extend_from_slice(
                                                   & field.as_ref()
                                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(| |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                                            .map(|c| String::from("NULL"))
                                                            .collect::<Vec<_>>())},
                                                   | some| {Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(&some)?))})?
                                               );
                                            } else {
                                                <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("DEFAULT, "));
                                            }

                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T> 
                                // TODO Option wrapping
                                    quote!(
                                         <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("?, "));
                                         params.extend_from_slice(
                                                   &entity
                                                    . #rust_field_ident .as_ref()
                                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(| |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                                    .map(|c| String::from("NULL")).collect::<Vec<_>>())},
                                                   | some| { Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(some)?))})?
                                                );
                                           )
                                },

                                1 if !field.preselect => { // Option<T> selectable 
                                    quote!(
                                        if let Some(field) = &entity. #rust_field_ident {
                                            <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("?, "));
                                             params.extend_from_slice(
                                                &<#rust_type_ident as toql::key::Key>::params(
                                                            &<#rust_type_ident as toql::key::Key>::get_key(field)?));
                                        } else {
                                              <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("DEFAULT, "));
                                        }                                     
                                    )
                                },
                                _ => { // T
                                    quote!(
                                        <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|  insert_stmt.push_str("?, "));
                                        params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(&entity. #rust_field_ident)?));
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
            let insert_statement = format!(
                "INSERT {{}}INTO {} ({{}}) VALUES ",
                self.sql_table_name 
            );
            let insert_columns_code = &self.insert_columns_code;

            let role_test = if self.insdel_roles.is_empty() {
                        quote!()
                    } else {
                        let roles = &self.insdel_roles;
                        quote!(
                            toql::query::assert_roles(roles, &[ #(String::from(#roles)),* ].iter().cloned().collect())
                            .map_err(|e|toql::error::ToqlError::SqlBuilderError(toql::sql_builder::SqlBuilderError::RoleRequired(e)))?;
                        
                    )};
            
            let optional_insert_duplicate_impl = if self.duplicate {
                quote!( impl toql::mutate::InsertDuplicate for #struct_ident {})

            } else {
                quote!()
            };

            quote! {
                #optional_insert_duplicate_impl
                
                impl<'a, T: toql::mysql::mysql::prelude::GenericConnection + 'a> toql::mutate::Insert<'_, #struct_ident> for toql::mysql::MySql<'a,T> {

                    type error = toql::mysql::error::ToqlMySqlError;

                     fn insert_many_sql<Q : std::borrow::Borrow<#struct_ident>>(entities: &[Q], strategy: toql::mutate::DuplicateStrategy, roles: &std::collections::HashSet<String>)
                     -> Result<Option<(String, Vec<String>)>, toql :: mysql::error:: ToqlMySqlError>
                     {
                            #role_test

                            if entities.is_empty() {
                                return Ok(None);
                            }

                            let mut params :Vec<String>= Vec::new();
                            let mut columns :Vec<String>= Vec::new();

                             let ignore = if let toql::mutate::DuplicateStrategy::Skip = strategy {
                                "IGNORE "
                            } else {
                                ""
                            };


                            #(#insert_columns_code)*
                            
                            let mut insert_stmt = format!( #insert_statement, ignore, columns.join(", "));

                           

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
                            if  let toql::mutate::DuplicateStrategy::Update = strategy {
                                insert_stmt.push_str(" ON DUPLICATE UPDATE");
                            };
                            Ok(Some((insert_stmt, params)))
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
