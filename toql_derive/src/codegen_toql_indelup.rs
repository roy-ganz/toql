/*
* Generation functions for toql derive
*
*/

use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use syn::Ident;

use heck::SnakeCase;

pub(crate) struct GeneratedToqlIndelup<'a> {
    struct_ident: &'a Ident,
    sql_table_ident: Ident,

    insert_columns: Vec<String>,
    insert_params_code: Vec<proc_macro2::TokenStream>,

    delup_keys: Vec<String>,
    delup_key_params_code: Vec<proc_macro2::TokenStream>,

    update_set_code: Vec<proc_macro2::TokenStream>,
}

impl<'a> GeneratedToqlIndelup<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedToqlIndelup {
        let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        let sql_table_ident = Ident::new(
            &toql.table.clone().unwrap_or(renamed_table),
            Span::call_site(),
        );

        GeneratedToqlIndelup {
            struct_ident: &toql.ident,
            sql_table_ident: sql_table_ident,
            insert_columns: Vec::new(),
            insert_params_code: Vec::new(),

            delup_keys: Vec::new(),
            delup_key_params_code: Vec::new(),

            update_set_code: Vec::new(),
        }
    }

    fn add_insert_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        if !field.merge.is_empty() || field.skip_inup || field.sql.is_some() {
            return;
        }

        let field_name = field.ident.as_ref().unwrap().to_string();

        // Regular field
        if field.sql_join.is_empty() {
            let field_ident = field.ident.as_ref().unwrap();
            let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
            self.insert_columns.push(sql_column);
            let options = field.number_of_options();
            let unwrap_null =
                        // Option<Option<T>> (toql selectable of nullable column)
                        if  options == 2 {
                            quote!(
                                .as_ref()
                                .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                .as_ref()
                                .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                            )
                        }
                        // Option<T>  (nullable column)
                        else if options == 1 && field.preselect {
                            quote!(
                                    .as_ref()
                                    .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                            )
                        }
                        // Option<T>  (toql selectable)
                        else if options == 1 { quote!(
                            .as_ref().ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                        ) } else {
                            quote!()
                        };

            let params = quote!( params.push( entity . #field_ident  #unwrap_null .to_string().to_owned()); );

            self.insert_params_code.push(params);
        }
        // Join field
        else {
            let field_ident = field.ident.as_ref().unwrap();
            for j in &field.sql_join {
                let auto_self_key = crate::util::rename(&field_ident.to_string(), &toql.columns);
                let self_column = j.this.as_ref().unwrap_or(&auto_self_key);
                self.insert_columns.push(self_column.to_owned());

                let other_field =
                    Ident::new(&j.other.to_string().to_snake_case(), Span::call_site());
                self.insert_params_code.push(
                      match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                        quote!(
                                                params.push(entity
                                                    . #field_ident .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    .as_ref()
                                                    . map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                    quote!(
                                            params.push(entity. #field_ident
                                                .as_ref(). map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                                params.push(entity. #field_ident
                                                    .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    . #other_field .to_string());
                                    )
                                },
                                _ => { // T
                                    quote!(
                                        params.push(entity. #field_ident . #other_field .to_string());
                                    )
                                }
                            }
                );

                /*  // TODO joins
                if field._first_type() == "Option" {

                    self.insert_params_code.push( quote!( params.push(entity. #field_ident
                            .as_ref().map_or( String::from("null"), |e| e. #other_field .to_string().to_owned())); ));

                } else {
                    // String fields will not turn into owned string, so add to_owned()
                    self.insert_params_code
                        .push(quote!( params.push(entity. #field_ident . #other_field .to_string().to_owned()); ));
                } */
            }
        }
    }

    fn add_delup_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        if !field.merge.is_empty() || field.sql.is_some() {
            return;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field.ident.as_ref().unwrap().to_string();
        let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

        // Key field
        if field.delup_key {
            // Option<key> (Toql selectable)
            // Keys for insert and delete may never be null
            if field._first_type() == "Option" {
                self.delup_key_params_code.push( quote!(
                    params.push(entity. #field_ident. as_ref()
                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?.to_string().to_owned() );
                ));
            } else {
                self.delup_key_params_code
                    .push(quote!(params.push(entity. #field_ident.to_string().to_owned()); ));
            }
            self.delup_keys
                .push(field.ident.as_ref().unwrap().to_string());
        }
        // Field is not skipped for update
        else if !field.skip_inup {
            // Regular field
            if field.sql_join.is_empty() {
                let set_statement = format!("{{}}.{} = ?, ", &sql_column);

                // Option<T>, <Option<Option<T>>
                if field._first_type() == "Option" && !field.preselect {
                    let unwrap_null = if 2 == field.number_of_options() {
                        quote!(.as_ref().map_or(String::from("NULL"), |x| x.to_string()))
                    } else {
                        quote!()
                    };

                    self.update_set_code.push(quote!(
                        if entity. #field_ident .is_some() {
                            update_stmt.push_str( &format!(#set_statement, alias));
                            params.push(entity . #field_ident .as_ref().unwrap() #unwrap_null .to_string() .to_owned());
                        }
                    ));
                }
                // T, Option<T> (nullable column)
                else {
                    let unwrap_null = if 1 == field.number_of_options() {
                        quote!(.map_or(String::from("NULL"), |x| x.to_string()))
                    } else {
                        quote!()
                    };
                    self.update_set_code.push(quote!(
                    update_stmt.push_str( &format!(#set_statement, alias));
                    params.push( entity . #field_ident #unwrap_null .to_string() .to_owned());
                        ));
                }
            }
            // Join Field
            else {
                for j in &field.sql_join {
                    let auto_self_key =
                        crate::util::rename(&field_ident.to_string(), &toql.columns);
                    let self_column = j.this.as_ref().unwrap_or(&auto_self_key);
                    let other_field =
                        Ident::new(&j.other.to_string().to_snake_case(), Span::call_site());
                    let set_statement = format!("{{}}.{} = ?, ", &self_column);

                    self.update_set_code.push(
                            match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                    quote!(
                                        if entity. #field_ident .is_some() {
                                            update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push(entity. #field_ident
                                                    .as_ref()
                                                    .unwrap()
                                                    .as_ref()
                                                    .map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                        }
                                    )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                    quote!(
                                            update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push(entity. #field_ident
                                                .as_ref(). map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                            if entity. #field_ident .is_some() {
                                                update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(entity. #field_ident
                                                    .as_ref().unwrap(). #other_field .to_string());
                                            }
                                    )
                                },
                                _ => { // T
                                    quote!(
                                        update_stmt.push_str( &format!(#set_statement, alias));
                                        params.push(entity. #field_ident . #other_field .to_string());
                                    )
                                }
                            }
                    );
                }
            }
        }
    }

    pub(crate) fn add_indelup_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        self.add_insert_field(toql, field);
        self.add_delup_field(toql, field);
    }
}
impl<'a> quote::ToTokens for GeneratedToqlIndelup<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let delup_key_comparison = self
            .delup_keys
            .iter()
            .map(|k| format!("{{alias}}.{} = ?", k))
            .collect::<Vec<String>>()
            .join(" AND ");

        let update_set_code = &self.update_set_code;

        let insert_params_code = &self.insert_params_code;
        let delup_key_params_code = &self.delup_key_params_code;

        let mods = if self.delup_keys.is_empty() {
            quote!( /* Skipped code generation, because #[toql(delup_key)] is missing */ )
        /*  quote_spanned! {
            struct_ident.span() =>
            compile_error!( "cannot find key(s) to delete and update: add `#[toql(delup_key)]` to at the field(s) in your struct that are the primary key in your table");
        } */
        } else {
            let sql_table_name = &self.sql_table_ident.to_string();
            // let update_one_statement = format!("UPDATE {} {{alias}} SET ", self.sql_table_ident);

            let insert_cols = format!(
                " ({})",
                self.insert_columns
                    .iter()
                    .map(|_v| "?".to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            );
            let insert_statement = format!(
                "INSERT INTO {} ({}) VALUES",
                self.sql_table_ident,
                self.insert_columns.join(",")
            );

            /*  let update_where_statement = format!(" WHERE {}", delup_key_comparison);
            let delete_one_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE {}",
                self.sql_table_ident, delup_key_comparison
            ); */
            let delete_many_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE ",
                self.sql_table_ident
            );

            quote! {
                impl<'a> toql::indelup::Indelup<'a, #struct_ident> for #struct_ident {


                   /*   fn insert_one_sql(entity: & #struct_ident) -> toql::error::Result<(String, Vec<String>)> {
                        Self::insert_many_sql(std::iter::once(entity))
                    } */

                     fn insert_many_sql<I>(entities: I)-> toql::error::Result<(String, Vec<String>)>
                     where I: IntoIterator<Item=&'a #struct_ident> + 'a
                     {


                            let mut params= Vec::new();
                            let mut insert_stmt = String::from( #insert_statement);

                            for entity in entities {
                                insert_stmt.push_str( #insert_cols );
                                #(#insert_params_code)*
                            }
                            Ok((insert_stmt, params))
                    }

                   /*  fn update_one_sql(  entity: & #struct_ident)  -> toql::error::Result<(String, Vec<String>)>
                    {
                        let alias= "t";
                        let mut params :Vec<String> = Vec::new();
                        let mut update_stmt = format!( #update_one_statement, alias = alias);

                        #(#update_set_code)*

                        update_stmt.pop(); // Remove trailing ", "
                        update_stmt.pop();

                        update_stmt.push_str( &format!(#update_where_statement, alias = alias));

                        // If no data to update then skip SQL update and return 1 row done
                         if params.is_empty() {
                            return Ok((String::from("-- Nothing to update"), params));
                        }

                        #(#delup_key_params_code)*

                        Ok((update_stmt, params))

                    } */
                    fn update_many_sql<I>(entities:I) -> toql::error::Result<(String, Vec<String>)>
                    where I: IntoIterator<Item=&'a #struct_ident> + 'a + Clone
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;

                        // Generate  join
                        for (i, entity) in entities.clone().into_iter().enumerate() {
                            let alias =  &format!("t{}", i);
                            if first {
                                first = false;
                            } else {
                                update_stmt.push_str("INNER JOIN ");
                            }
                            update_stmt.push_str( &format!("{} {} ", #sql_table_name, alias)) ;
                        }

                        // Generate SET
                         update_stmt.push_str("SET ");
                         for (i, entity) in entities.clone().into_iter().enumerate() {
                                let alias = &format!("t{}", i);
                                 #(#update_set_code)*
                         }
                         update_stmt.pop(); // Remove trailing ", "
                         update_stmt.pop();

                         if params.is_empty() {
                            return Ok((String::from("-- Nothing to update"), params));
                        }
                        update_stmt.push_str(" WHERE ");
                        let mut first = true;
                         for (i, entity) in entities.clone().into_iter().enumerate() {
                            let alias = &format!("t{}", i);
                            if first {
                                first = false;
                            } else {
                                update_stmt.push_str(" AND ");
                            }
                            update_stmt.push_str( &format!(#delup_key_comparison, alias = alias));

                            #(#delup_key_params_code)*
                         }

                        Ok((update_stmt, params))

                    }
                  /*   fn delete_one_sql(  entity: & #struct_ident) -> toql::error::Result<(String, Vec<String>)>
                    {
                        let alias="t";
                        let mut params :Vec<String>= Vec::new();
                        let delete_stmt = format!(#delete_one_statement, alias = alias);

                        #(#delup_key_params_code)*

                        Ok((delete_stmt, params))
                     } */

                        fn delete_many_sql<I>(entities: I) -> toql::error::Result<(String, Vec<String>)>
                        where I:  IntoIterator<Item=&'a #struct_ident> +'a
                        {
                            let alias= "t";
                            let mut delete_stmt =format!(#delete_many_statement, alias = alias);

                            let mut params :Vec<String>= Vec::new();
                            let mut first = true;
                            for entity in entities {
                                    if first {
                                        first = false;
                                    }else {
                                       delete_stmt.push_str(" OR ");
                                    }
                                   delete_stmt.push('(');
                                   delete_stmt.push_str( &format!( #delup_key_comparison, alias = alias));
                                   delete_stmt.push(')');

                                  #(#delup_key_params_code)*
                            }

                            Ok((delete_stmt, params))
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
