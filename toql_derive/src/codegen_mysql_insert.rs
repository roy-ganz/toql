/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use syn::Ident;

pub(crate) struct GeneratedMysqlInsert<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,

    insert_columns_code: Vec<TokenStream>,

    insert_params_code: Vec<TokenStream>,

}

impl<'a> GeneratedMysqlInsert<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlInsert {
        GeneratedMysqlInsert {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            insert_columns_code: Vec::new(),

            insert_params_code: Vec::new(),

           
        }
    }

     pub(crate) fn add_insert_field(&mut self, field: &crate::sane::Field) {
        if field.skip_mut {
            return;
        }
        let rust_field_name = &field.rust_field_name;
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

                let unwrap_null = match field.number_of_options {
                    2 => {
                        // Option<Option<T>> (toql selectable of nullable column)
                        quote!(
                            .as_ref()
                            .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?
                            .as_ref()
                            .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                        )
                    }
                    1 if field.preselect => {
                        // Option<T>  (nullable column)
                        quote!(
                            .as_ref()
                            .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                        )
                    }
                    1 if !field.preselect => {
                        // Option<T>  (toql selectable)
                        quote!(
                        .as_ref().ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?
                        )
                    }
                    _ => quote!(),
                };

                let params = quote!( params.push( entity . #rust_field_ident  #unwrap_null .to_string().to_owned()); );

                self.insert_params_code.push(params);
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

                self.insert_params_code.push(
                      match field.number_of_options  {
                                2 => { // Option<Option<T>>
                                        quote!(
                                                params.extend_from_slice(
                                                    &entity
                                                    . #rust_field_ident .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?
                                                   .as_ref()
                                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(| |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                                            .map(|c| String::from("NULL"))
                                                            .collect::<Vec<_>>())},
                                                   | some| {Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(some)?))})?

                                                );

                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                // TODO Option wrapping
                                    quote!(
                                         params.extend_from_slice(
                                                   &entity
                                                    . #rust_field_ident .as_ref()
                                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(| |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                                    .map(|c| String::from("NULL")).collect::<Vec<_>>())},
                                                   | some| { Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(some)?))})?
                                                );
                                           )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                      params.extend_from_slice(
                                          &<#rust_type_ident as toql::key::Key>::params(
                                                    &<#rust_type_ident as toql::key::Key>::get_key( entity
                                                    . #rust_field_ident .as_ref()
                                                     .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?)?
                                          )
                                      );
                                    )
                                },
                                _ => { // T
                                    quote!(
                                        params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(&entity. #rust_field_ident)?));
                                   )
                                }
                            }
                );
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

      

        let insert_params_code = &self.insert_params_code;
        

        
        let mods = {
            let insert_statement = format!(
                "INSERT {{}}INTO {} ({{}}) VALUES",
                self.sql_table_name 
            );
            let insert_columns_code = &self.insert_columns_code;

            quote! {

                
                impl<'a> toql::mysql::insert::Insert<'a, #struct_ident> for #struct_ident {

                     fn insert_many_sql<I>(entities: I, strategy: toql::mysql::insert::DuplicateStrategy)-> toql::error::Result<Option<(String, Vec<String>)>>
                     where I: IntoIterator<Item=&'a #struct_ident> + 'a
                     {


                            let mut params :Vec<String>= Vec::new();
                            let mut columns :Vec<String>= Vec::new();

                             let ignore = if let toql::mysql::insert::DuplicateStrategy::Skip = strategy {
                                "IGNORE "
                            } else {
                                ""
                            };


                            #(#insert_columns_code)*
                            let placeholder = format!(" ({})",columns.iter().map(|_| "?").collect::<Vec<&str>>().join(","));
                            let mut insert_stmt = format!( #insert_statement, ignore, columns.join(","));

                            for entity in entities {
                                // #(#insert_placeholder_code)*
                                insert_stmt.push_str( &placeholder );
                                #(#insert_params_code)*
                            }
                             if params.is_empty() {
                                return Ok(None);
                            }
                            if  let toql::mysql::insert::DuplicateStrategy::Update = strategy{
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
