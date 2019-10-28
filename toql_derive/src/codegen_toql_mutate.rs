/*
* Generation functions for toql derive
*
*/

use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use syn::Ident;

use heck::SnakeCase;

pub(crate) struct GeneratedToqlMutate<'a> {
    struct_ident: &'a Ident,
    sql_table_ident: Ident,

    insert_columns_code: Vec<proc_macro2::TokenStream>,
    
    insert_params_code: Vec<proc_macro2::TokenStream>,

    //keys: Vec<String>,
    key_params_code: Vec<proc_macro2::TokenStream>,
    key_columns_code: Vec<proc_macro2::TokenStream>,

    update_set_code: Vec<proc_macro2::TokenStream>,
    diff_set_code: Vec<proc_macro2::TokenStream>,

    diff_merge_code: Vec<proc_macro2::TokenStream>,

  
}

impl<'a> GeneratedToqlMutate<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedToqlMutate {
        let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        let sql_table_ident = Ident::new(
            &toql.table.clone().unwrap_or(renamed_table),
            Span::call_site(),
        );

        GeneratedToqlMutate {
            struct_ident: &toql.ident,
            sql_table_ident: sql_table_ident,
            insert_columns_code: Vec::new(),
          
            insert_params_code: Vec::new(),

            key_columns_code: Vec::new(),
            key_params_code: Vec::new(),

            update_set_code: Vec::new(),
            diff_set_code: Vec::new(),

            diff_merge_code: Vec::new(),

        }
    }

    


    fn add_insert_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        if !field.merge.is_empty() || field.skip_mut || field.sql.is_some() {
            return;
        }

        let field_name = field.ident.as_ref().unwrap().to_string();

        // Regular field
        if field.join.is_none() {
            let field_ident = field.ident.as_ref().unwrap();
                       
            let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
            self.insert_columns_code.push( quote!(  columns.push(String::from(#sql_column));));
            
           
            let unwrap_null =
                match field.number_of_options() {
                    2 =>  { // Option<Option<T>> (toql selectable of nullable column)
                        quote!( 
                            .as_ref()
                            .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                            .as_ref()
                            .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                        )
                        },
                        1 if field.preselect => {  // Option<T>  (nullable column)
                            quote!(
                                .as_ref()
                                .map_or(String::from("NULL"), |x| x.to_string().to_owned())
                            )
                        },
                            1 if !field.preselect => {  // Option<T>  (toql selectable)
                                quote!(
                            .as_ref().ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                            )
                            },
                            _ =>  quote!()


                };
                       

            let params = quote!( params.push( entity . #field_ident  #unwrap_null .to_string().to_owned()); );

            self.insert_params_code.push(params);
        }
        // Join field
        // Get the sql column value from the related struct
        // Because we don't know whether the struct field is Option<>
        // We convert in any case to Option and unwrap

        else if field.join.is_some(){
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = field.first_non_generic_type().unwrap();
            let joined_struct_ident = field.first_non_generic_type();
           // let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_name), &toql.columns)];
            /* let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() { 
                field.join.as_ref().unwrap().this_columns.as_ref() }
                else {
                    &default_self_columns
                }; */
                 
             let default_column_format = format!("{}_{{}}", field_ident);
             let match_translation  = field.join.as_ref().unwrap().columns.iter()
                        .map(|column| { 
                            let tc = &column.this; let oc = &column.other;
                            quote!( #oc => #tc,)
                        })
                        .collect::<Vec<_>>();

                self.insert_columns_code.push( 
                    quote!(
                     
                         for other_column in <#joined_struct_ident as toql::key::Key>::columns() {
                            let default_self_column = format!(#default_column_format, other_column);
                                let self_column = match other_column.as_str() {
                                    #(#match_translation)*
                                    _ => &default_self_column
                                    };
                            columns.push(self_column.to_owned());
                         }
                            //format!("({}{}{} IS NOT NULL)",sql_alias, if sql_alias.is_empty() { "" } else { "." }, self_column)
                    
                    )
                );

                self.insert_params_code.push(
                      match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                        quote!(
                                                params.extend_from_slice(
                                                    entity
                                                    . #field_ident .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                   .as_ref()
                                                   .map_or_else::<Result<String,toql::error::ToqlError>,_>(| none |{ Ok(<#field_type as toql::key::Key>::columns()iter().map(|c| String::from("NULL").collect::<Vec<_>()))},
                                                   | some| {<#field_type as toql::key::Key>::params(some)})?
                                                   /*  .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(<#field_type as toql::key::Key>::get_key(e)?
                                                            .#column_index
                                                            .to_string())
                                                    })? */
                                                );

                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                // TODO Option wrapping
                                    quote!(
                                         params.extend_from_slice(
                                                    entity
                                                    . #field_ident .as_ref()
                                                   .map_or_else::<Result<String,toql::error::ToqlError>,_>(| none |{ Ok(<#field_type as toql::key::Key>::columns()iter().map(|c| String::from("NULL").collect::<Vec<_>()))},
                                                   | some| {<#field_type as toql::key::Key>::params(some)})?
                                                   /*  .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(<#field_type as toql::key::Key>::get_key(e)?
                                                            .#column_index
                                                            .to_string())
                                                    })? */
                                                );
                                           /*  params.push(entity. #field_ident
                                                .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(<#field_type as toql::key::Key>::get_key(e)?
                                                            .#column_index
                                                            .to_string())
                                                    })?
                                            ); */
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                      params.extend_from_slice(
                                          <#field_type as toql::key::Key>::params(
                                                    entity
                                                    . #field_ident .as_ref()
                                                     .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                          )?
                                      );
                                             /*    params.push(

                                                     <#field_type as toql::key::Key>::get_key(entity. #field_ident.as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    )?. #column_index  .to_string()); */
                                                     
                                    )
                                    /* quote!(
                                                params.push(
                                                    Option::from(entity. #field_ident
                                                    .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    . #other_field)
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#composite_field_name)))?
                                                     .to_string());
                                    ) */
                                },
                                _ => { // T
                                    quote!(
                                        params.extend_from_slice(<#field_type as toql::key::Key>::params(&entity. #field_ident)?);
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

            
            /* for (i, self_column) in self_columns.iter().enumerate() {
            /*     let auto_self_key = crate::util::rename(&format!("{}_id", field_name), &toql.columns);
                let self_column = j.this_column.as_ref().unwrap_or(&auto_self_key);
                self.insert_columns.push(self_column.to_owned());
                let default_other_column = crate::util::rename("id", &toql.columns);
                let other_field =
                    Ident::new(&j.other_column.as_ref().unwrap_or(&default_other_column), Span::call_site()); */
                
                self.insert_columns.push(self_column.to_owned());

                let column_index= syn::Index::from(i);
                self.insert_params_code.push(
                      match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                        quote!(
                                                params.push(entity
                                                    . #field_ident .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                   .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(<#field_type as toql::key::Key>::get_key(e)?
                                                            .#column_index
                                                            .to_string())
                                                    })?
                                                );
                                        )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                // TODO Option wrapping
                                    quote!(
                                            params.push(entity. #field_ident
                                                .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(<#field_type as toql::key::Key>::get_key(e)?
                                                            .#column_index
                                                            .to_string())
                                                    })?
                                            );
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                // TODO object tolerance on foreign field
                                //let composite_field_name = format!("{}.{}",&field_name , &other_field );
                                quote!(
                                                params.push(

                                                     <#field_type as toql::key::Key>::get_key(entity. #field_ident.as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    )?. #column_index  .to_string());
                                                     
                                    )
                                    /* quote!(
                                                params.push(
                                                    Option::from(entity. #field_ident
                                                    .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    . #other_field)
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#composite_field_name)))?
                                                     .to_string());
                                    ) */
                                },
                                _ => { // T
                                    quote!(
                                        params.push(<#field_type as toql::key::Key>::get_key(&entity. #field_ident)?. #column_index  .to_string());
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
        } */
    }

    fn add_delup_field(&mut self, toql: &Toql, field: &'a ToqlField) {

        // SQL code cannot be updated, skip field
        if field.sql.is_some() {
            return;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_type = field.first_non_generic_type().unwrap();
        let field_name = field.ident.as_ref().unwrap().to_string();
        let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

        // Key field
        if field.key {
            if field.join.is_none() {
            // Option<key> (Toql selectable)
            // Keys for insert and delete may never be null
            if field._first_type() == "Option" {
                self.key_params_code.push( quote!(
                    params.push(entity. #field_ident. as_ref()
                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?.to_string().to_owned() );
                ));
            } else {
                self.key_params_code
                    .push(quote!(params.push(entity. #field_ident.to_string().to_owned()); ));
            }

            // Add field to keys, struct may contain multiple keys (composite key) 
            self.key_columns_code
                .push( quote!(
                    keys.push(String::from(#field_name));
                ));
            } 
            // Join used as key field
            else {
                     let joined_struct_ident = field.first_non_generic_type();

          /*   let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_name), &toql.columns)];
            let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() { 
                field.join.as_ref().unwrap().this_columns.as_ref() }
                else {
                    &default_self_columns
                }; */
                  let default_column_format = format!("{}_{{}}", field_ident);
                let match_translation  = field.join.as_ref().unwrap().columns.iter()
                        .map(|column| { 
                            let tc = &column.this; let oc = &column.other;
                            quote!( #oc => #tc,)
                        })
                        .collect::<Vec<_>>();

                self.key_columns_code.push(
                    quote!(
                        keys.extend_from_slice( <#joined_struct_ident as toql::key::Key>::columns( ).iter()
                        .map(|other_column|{
                            let default_self_column = format!(#default_column_format, other_column);
                            match other_column {
                                #( #match_translation)*
                                _ => default_self_column

                            }}).collect::<Vec<_>>() 
                         );
                    )
                );
                    

                 if field.number_of_options() > 0 {
                        self.key_params_code.push( 
                            quote!(
                                 params.extend_from_slice( toql::key::Key::params( &entity . #field_ident
                                 .as_ref() .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                 )?);
                            )
                                   /*  params.push( toql::key::Key::get_key(
                                        entity . #field_ident.as_ref() .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                        
                                        )?. #key_index ); */
                            );
                       
                                
                        
                    } else {
                        self.key_params_code
                            .push(
                                    quote!(
                                        params.extend_from_slice( toql::key::Key::params( &entity . #field_ident)?);
                                        //params.push( toql::key::Key::get_key( &entity . #field_ident)?. #key_index .to_string().to_owned() );
                                    )
                                );
                    }

            
            /* for (i, self_column) in self_columns.iter().enumerate() {
                
                    let key_index = syn::Index::from(i);
                    // Keys for insert and delete may never be null
                    // TODO match 
                    if field.number_of_options() > 0 {
                        self.key_params_code.push( 
                            quote!(
                               
                                    params.push( toql::key::Key::get_key(
                                        entity . #field_ident.as_ref() .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                        
                                        )?. #key_index );
                            )
                        );
                                
                        
                    } else {
                        self.key_params_code
                            .push(
                                    quote!(
                                        params.exte
                                        params.push( toql::key::Key::get_key( &entity . #field_ident)?. #key_index .to_string().to_owned() );
                                    )
                                );
                    }

                    self.keys
                        .push(self_column.to_string());
                    }  */

            }


           // }
        }

        // Field is not skipped for update
         if !field.skip_mut {
            // Regular field
            if field.join.is_none() && field.merge.is_empty() {
                   
                let set_statement = format!("{{}}.{} = ?, ", &sql_column);

                // Option<T>, <Option<Option<T>>
                if field._first_type() == "Option" && !field.preselect {
                    let unwrap_null = if 2 == field.number_of_options() {
                        quote!(.as_ref().map_or(String::from("NULL"), |x| x.to_string()))
                    } else {
                        quote!()
                    };

                    // update statement
                    // Doesn't update primary key
                    if !field.key {
                        self.update_set_code.push(quote!(
                            if entity. #field_ident .is_some() {
                                update_stmt.push_str( &format!(#set_statement, alias));
                                params.push(entity . #field_ident .as_ref().unwrap() #unwrap_null .to_string() .to_owned());
                            }
                            ));
                    }
                    // diff statement
                    self.diff_set_code.push(quote!(
                        if entity. #field_ident .is_some()
                         && outdated. #field_ident  .as_ref().ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))? != entity. #field_ident .as_ref().unwrap()
                         {
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
                    //update statement
                    if !field.key {
                        self.update_set_code.push(quote!(
                        update_stmt.push_str( &format!(#set_statement, alias));
                        params.push( entity . #field_ident #unwrap_null .to_string() .to_owned());
                            ));
                    }

                    // diff statement
                    self.diff_set_code.push(quote!(
                        if outdated.  #field_ident != entity. #field_ident
                        {
                                update_stmt.push_str( &format!(#set_statement, alias));
                                 params.push( entity . #field_ident #unwrap_null .to_string() .to_owned());
                            
                        }
                    ));

                }
            }
            // Join Field
            else if field.join.is_some(){

                let default_column_format = format!("{}_{{}}", field_ident);
             let match_translation  = field.join.as_ref().unwrap().columns.iter()
                        .map(|column| { 
                            let tc = &column.this; let oc = &column.other;
                            quote!( #oc => #tc,)
                        })
                        .collect::<Vec<_>>();

                let add_columns_to_update_stmt = quote!(
                     for other_column in <#field_type as toql::key::Key>::columns() {
                        let default_self_column = format!(#default_column_format, other_column);
                        let self_column = match other_column {
                            #(#match_translation)*
                            _ => &default_self_column
                        };
                        update_stmt.push(self_column);
                    }
                );

            let  set_params_code = match  field.number_of_options()  {
                    2 => { // Option<Option<T>>
                                        quote!( params.extend_from_slice(
                                                    entity. #field_ident
                                                        .as_ref()
                                                        .unwrap()
                                                        .as_ref()
                                                   .map_or_else::<Result<String,toql::error::ToqlError>,_>(| none |{ Ok(<#field_type as toql::key::Key>::columns()iter().map(|c| String::from("NULL").collect::<Vec<_>()))},
                                                        | some| {<#field_type as toql::key::Key>::params(some)})?
                                                        
                                                    );
                                        )
                    },
                    1 if field.preselect => { // #[toql(preselect)] Option<T>
                                        quote!(
                                            params.extend_from_slice(
                                                    entity
                                                    . #field_ident .as_ref()
                                                   .map_or_else::<Result<String,toql::error::ToqlError>,_>(| none |{ Ok(<#field_type as toql::key::Key>::columns()iter().map(|c| String::from("NULL").collect::<Vec<_>()))},
                                                   | some| {<#field_type as toql::key::Key>::params(some)})?);
                                        )
                    }
                    1 if !field.preselect => { // Option<T>
                                        quote!(
                                             params.extend_from_slice( <#field_type as toql::key::Key>::params(
                                                        entity. #field_ident .as_ref().unwrap()));
                                        )
                    }
                    _ => { // T
                         quote!(
                                            //update_stmt.push_str( &format!(#set_statement, alias));
                                              #add_columns_to_update_stmt
                                            params.extend_from_slice(<#field_type as toql::key::Key>::params(&entity. #field_ident)?);
                         )
                    }



            };

                self.update_set_code.push(
                                match field.number_of_options()  {
                                    2 => { // Option<Option<T>>
                                        quote!(
                                            if entity. #field_ident .is_some() {
                                               
                                               #add_columns_to_update_stmt
                                                
                                               #set_params_code

                                                /* params.push(entity. #field_ident
                                                        .as_ref()
                                                        .unwrap()
                                                        .as_ref()
                                                          .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(toql::key::Key::get_key(e)?
                                                            .#join_key_index
                                                            .to_string())
                                                    })?
                                                        
                                                ); */
                                                        
                                            }
                                        )
                                        },
                                    1 if field.preselect => { // #[toql(preselect)] Option<T>
                                        quote!(
                                                 #add_columns_to_update_stmt
                                                
                                               
                                            #set_params_code
                                            /* params.push(entity. #field_ident
                                                    .as_ref().map_or::<Result<String,toql::error::ToqlError>,_>(
                                                        Ok(String::from("NULL")),
                                                        |e|{  Ok(toql::key::Key::get_key(e)? 
                                                        .#join_key_index 
                                                        .to_string())
                                                        })?
                                                    ); */
                                        )
                                    },

                                    1 if !field.preselect => { // Option<T>
                                        quote!(
                                                if entity. #field_ident .is_some() {
                                                    //update_stmt.push_str( &format!(#set_statement, alias));
                                                      #add_columns_to_update_stmt
                                                      #set_params_code
                                                   /*  params.push(
                                                        toql::key::Key::get_key( entity. #field_ident .as_ref().unwrap())? 
                                                        .#join_key_index .to_string()); */
                                                       
                                                     
                                                }
                                        )
                                    },
                                    _ => { // T
                                        quote!(
                                            //update_stmt.push_str( &format!(#set_statement, alias));
                                              #add_columns_to_update_stmt
                                                #set_params_code
                                            //params.push( toql::key::Key::get_key(&entity. #field_ident)? . #join_key_index .to_string());
                                        )
                                    }
                                }
                            
                       


                 /*  let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_name), &toql.columns)];
            let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() { 
                field.join.as_ref().unwrap().this_columns.as_ref() }
                else {
                    &default_self_columns
                };
              //  self.key_columns_code.push( quote!( columns.extend_from_slice(&<#field_type as toql::key::Key>::columns());));

                let default_other_columns= vec![crate::util::rename("id", &toql.columns)];
            let other_columns =  if !field.join.as_ref().unwrap().other_columns.is_empty() { 
                field.join.as_ref().unwrap().other_columns.as_ref() }
                else {
                    &default_other_columns
                };
            self_columns.iter().zip(other_columns).enumerate().for_each( |(i,(self_column, other_column))| {

                    let set_statement = format!("{{}}.{} = ?, ", &self_column);
                    //let other_field ="GET_KEY"; // Replace

                    let join_key_index= syn::Index::from(i);

                    // update code
                    if !field.key {
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
                                                          .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(toql::key::Key::get_key(e)?
                                                            .#join_key_index
                                                            .to_string())
                                                    })?
                                                         /* .map_or(String::from("NULL"), |e| {
                                                            toql::key::Key::get_key(e)
                                                                .map(|e| e.  #join_key_index .to_string())
                                                                .unwrap_or(String::from("NULL"))
                                                        }) */
                                                );
                                                        //.map_or(String::from("NULL"), |e| toql::key::Key::get_key(e)? . #join_key_index .to_string()));
                                            }
                                        )
                                        },
                                    1 if field.preselect => { // #[toql(preselect)] Option<T>
                                        quote!(
                                                update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(entity. #field_ident
                                                    .as_ref().map_or::<Result<String,toql::error::ToqlError>,_>(
                                                        Ok(String::from("NULL")),
                                                        |e|{  Ok(toql::key::Key::get_key(e)? 
                                                        .#join_key_index 
                                                        .to_string())
                                                        })?
                                                    );
                                        )
                                    },

                                    1 if !field.preselect => { // Option<T>
                                        quote!(
                                                if entity. #field_ident .is_some() {
                                                    update_stmt.push_str( &format!(#set_statement, alias));
                                                    params.push(
                                                        toql::key::Key::get_key( entity. #field_ident .as_ref().unwrap())? 
                                                        .#join_key_index .to_string());
                                                }
                                        )
                                    },
                                    _ => { // T
                                        quote!(
                                            update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push( toql::key::Key::get_key(&entity. #field_ident)? . #join_key_index .to_string());
                                        )
                                    }
                                }
                        );
                    } */
                    );
                    // diff code
                    let join_key_index= syn::Index::from(self.key_params_code.len()-1);
                    self.diff_set_code.push(
                            match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                    quote!(
                                        if entity. #field_ident .is_some() 
                                        && 
                                         entity. #field_ident 
                                                    .as_ref() .unwrap()
                                                    .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                                Ok(toql::key::Key::get_key(e)? . #join_key_index .to_string())
                                                    })?
                                        !=  outdated. #field_ident 
                                        .as_ref() .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?   
                                        .as_ref().map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                            Ok(toql::key::Key::get_key(e)? . #join_key_index .to_string())
                                                                
                                         })?
                                        {
                                            #add_columns_to_update_stmt
                                            #set_params_code

                                          /*   update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push(entity. #field_ident
                                                    .as_ref()
                                                    .unwrap()
                                                    .as_ref()
                                                     .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                        Ok(toql::key::Key::get_key(e)?
                                                           . #join_key_index
                                                            .to_string())
                                                    })?
                                                     /* .map_or(String::from("NULL"), |e| {
                                                            toql::key::Key::get_key(e)
                                                                .map(|e| e.  #join_key_index .to_string())
                                                                .unwrap_or(String::from("NULL"))
                                                        }) */
                                            ); */
                                                    //.map_or(String::from("NULL"), |e| toql::key::Key::get_key(e)? . #join_key_index .to_string()));
                                        }
                                    )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                    quote!(
                                            if    entity. #field_ident 
                                                    .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                                Ok(toql::key::Key::get_key(e)? . #join_key_index .to_string())
                                                    })?
                                                !=  outdated. #field_ident 
                                                    .as_ref()
                                                    .map_or::<Result<String,toql::error::ToqlError>,_>(Ok(String::from("NULL")), |e| {
                                                                Ok(toql::key::Key::get_key(e)? . #join_key_index .to_string())
                                                })?
                                            {
                                                #add_columns_to_update_stmt
                                            #set_params_code
                                                /* update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(entity. #field_ident
                                                    .as_ref(). map_or::<Result<String,toql::error::ToqlError>,_>(
                                                        Ok(String::from("NULL")), 
                                                        |e|  { Ok(toql::key::Key::get_key(e)? . #join_key_index .to_string())
                                                        })?
                                                ); */
                                            }
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                            if entity. #field_ident .is_some() 
                                            && toql::key::Key::get_key(entity .  #field_ident.as_ref() .unwrap())?  
                                             !=  toql::key::Key::get_key(outdated .  #field_ident.as_ref()
                                             .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?   
                                              )? 
                                               
                                            {
                                                #add_columns_to_update_stmt
                                            #set_params_code
                                               /*  update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(toql::key::Key::get_key(entity. #field_ident
                                                    .as_ref().unwrap())? . #join_key_index.to_string()); */
                                            }
                                    )
                                },
                                _ => { // T
                               
                                    quote!(
                                         if toql::key::Key::get_key(&outdated. #field_ident)? !=  toql::key::Key::get_key(&entity. #field_ident)? {
                                             #add_columns_to_update_stmt
                                            #set_params_code
                                           /*  update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push( toql::key::Key::get_key(&entity. #field_ident)?. #join_key_index ); */
                                         }
                                    )
                                }
                            }
                    );
                // });
            } 
         
            // merge fields
            else {

                let merge_type_ident = field.first_non_generic_type();
                
                let optional =  field.number_of_options() > 0 ;
                
                let optional_unwrap = if optional { quote!( .unwrap())} else {quote!()};
                let optional_if = if optional { quote!(if entity .  #field_ident  .is_some() )} else {quote!()};
                let optional_ok_or = if optional { quote!(  .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?)} else {quote!()};
                
                self.diff_merge_code.push( quote!(
                        let mut insert : Vec<& #merge_type_ident> = Vec::new();
                        let mut diff : Vec<(& #merge_type_ident, & #merge_type_ident)> = Vec::new();
                        let mut delete : Vec<& #merge_type_ident> = Vec::new();

                        for (outdated, entity) in entities.clone() {
                            #optional_if {
                                let mut delta  = toql::diff::collections_delta(std::iter::once((outdated. #field_ident  .as_ref() #optional_ok_or, entity. #field_ident .as_ref()  #optional_unwrap )))?;
                            
                                insert.append(&mut delta.0);
                                diff.append(&mut delta.1);
                                delete.append(&mut delta.2);
                            }
                        }

                        let insert_sql =  #merge_type_ident::insert_many_sql(insert)?;
                        let diff_sql =  #merge_type_ident::shallow_diff_many_sql(diff)?; // shallow_diff (Exclude merge tables)
                        let delete_sql =  #merge_type_ident::delete_many_sql(delete)?;

                        if let Some( s) = insert_sql {
                            sql.push(s);
                        }
                        if let Some( s) = diff_sql {
                            sql.push(s);
                        }
                        if let Some( s) = delete_sql {
                            sql.push(s);
                        }

                ));



            }
        }
        
    }
    

    pub(crate) fn add_mutate_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        self.add_insert_field(toql, field);
        self.add_delup_field(toql, field);
    }
}
impl<'a> quote::ToTokens for GeneratedToqlMutate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;
/* 
        let key_comparison = self
            .keys
            .iter()
            .map(|k| format!("{{alias}}.{} = ?", k))
            .collect::<Vec<String>>()
            .join(" AND ");
 */
        let update_set_code = &self.update_set_code;
         let diff_set_code = &self.diff_set_code;

        let insert_params_code = &self.insert_params_code;
        let key_params_code = &self.key_params_code;

        let diff_merge_code = &self.diff_merge_code;

        // Generate modules if there are keys available
        let mods = if self.key_columns_code.is_empty() {
            quote!( /* Skipped code generation, because #[toql(key)] is missing */ )
        /*  quote_spanned! {
            struct_ident.span() =>
            compile_error!( "cannot find key(s) to delete and update: add `#[toql(key)]` to at the field(s) in your struct that are the primary key in your table");
        } */
        } else {
            let sql_table_name = &self.sql_table_ident.to_string();
            // let update_one_statement = format!("UPDATE {} {{alias}} SET ", self.sql_table_ident);

           /*  let insert_cols = format!(
                " ({})",
                self.insert_columns
                    .iter()
                    .map(|_v| "?".to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ); */
            let insert_statement = format!(
                "INSERT INTO {} ({{}}) VALUES",
                self.sql_table_ident //,
               // self.insert_columns.join(",")
            );
            let insert_columns_code = &self.insert_columns_code;
            

            /*  let update_where_statement = format!(" WHERE {}", key_comparison);
            let delete_one_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE {}",
                self.sql_table_ident, key_comparison
            ); */
            let delete_many_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE ",
                self.sql_table_ident
            );

            let key_columns_code = &self.key_columns_code;
             
            quote! {

               
                

                impl<'a> toql::mutate::Mutate<'a, #struct_ident> for #struct_ident {



                     fn insert_many_sql<I>(entities: I)-> toql::error::Result<Option<(String, Vec<String>)>>
                     where I: IntoIterator<Item=&'a #struct_ident> + 'a
                     {


                            let mut params :Vec<String>= Vec::new();
                            let mut columns :Vec<String>= Vec::new();
                           


                            #(#insert_columns_code)*
                            let placeholder = columns.iter().map(|| "?").collect::<String>().join(",");
                            let mut insert_stmt = format!( #insert_statement, columns.join(","));

                            for entity in entities {
                                // #(#insert_placeholder_code)*
                                insert_stmt.push_str( placeholder );
                                #(#insert_params_code)*
                            }
                             if params.is_empty() {
                                return Ok(None);
                            }
                            Ok(Some((insert_stmt, params)))
                    }

                  
                    fn update_many_sql<I>(entities:I) -> toql::error::Result<Option<(String, Vec<String>)>>
                    where I: IntoIterator<Item=&'a #struct_ident> + 'a + Clone
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;
                        let mut keys: Vec<String> = Vec::new();

                         #(#key_columns_code)*
                       
                        

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
                            return Ok(None);
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
                            let key_comparison = keys.iter()
                                .map(|key| format!("{}.{} = ?", alias, key))
                                .collect::<Vec<String>>()
                                .join(" AND ");

                            update_stmt.push_str(&key_comparison);

                            #(#key_params_code)*
                         }

                        Ok(Some((update_stmt, params)))

                    }
                    fn shallow_diff_many_sql<I>(entities:I) -> toql::error::Result<Option<(String, Vec<String>)>>
                    where I: IntoIterator<Item=(&'a #struct_ident, &'a #struct_ident)> + 'a + Clone
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut keys: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;

                        #(#key_columns_code)*
                       

                        // Generate  join
                        for (i, (outdated, entity)) in entities.clone().into_iter().enumerate() {
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
                         for (i, (outdated, entity)) in entities.clone().into_iter().enumerate() {
                                let alias = &format!("t{}", i);
                                 #(#diff_set_code)*
                         }
                         update_stmt.pop(); // Remove trailing ", "
                         update_stmt.pop();

                         if params.is_empty() {
                            return Ok(None);
                        }
                        update_stmt.push_str(" WHERE ");
                        let mut first = true;
                         for (i, (outdated, entity)) in entities.clone().into_iter().enumerate() {
                            let alias = &format!("t{}", i);
                            if first {
                                first = false;
                            } else {
                                update_stmt.push_str(" AND ");
                            }
                            let key_comparison = keys.iter()
                                .map(|key| format!("{}.{} = ?", alias, key))
                                .collect::<Vec<String>>()
                                .join(" AND ");
                            update_stmt.push_str(&key_comparison);


                            #(#key_params_code)*
                         }

                        if params.is_empty() {
                            return Ok(None);
                        }
                        Ok(Some((update_stmt, params)))

                    }
                    fn diff_many_sql<I> (entities: I) -> toql::error::Result<Option<Vec<(String,Vec<String>)>>>
                    where I: IntoIterator<Item = (&'a #struct_ident, &'a #struct_ident)>  +'a +  Clone,
                    {
                        
                        let mut sql: Vec<(String, Vec<String>)> = Vec::new();

                        let update = #struct_ident::shallow_diff_many_sql(entities.clone())?;
                        if update.is_some() {
                            sql.push(update.unwrap());
                        }

                            #(#diff_merge_code)*

                            if sql.is_empty() {
                                return Ok(None);
                            }

                            Ok(Some(sql))

                    }
                    


                  /*   fn delete_one_sql(  entity: & #struct_ident) -> toql::error::Result<(String, Vec<String>)>
                    {
                        let alias="t";
                        let mut params :Vec<String>= Vec::new();
                        let delete_stmt = format!(#delete_one_statement, alias = alias);

                        #(#key_params_code)*

                        Ok((delete_stmt, params))
                     } */

                        fn delete_many_sql<I>(entities: I) -> toql::error::Result<Option<(String, Vec<String>)>>
                        where I:  IntoIterator<Item=&'a #struct_ident> +'a
                        {
                            let alias= "t";
                            let mut delete_stmt =format!(#delete_many_statement, alias = alias);

                            let mut params :Vec<String>= Vec::new();
                            let mut keys :Vec<String>= Vec::new();
                            let mut first = true;
                             
                             #(#key_columns_code)*

                              let key_comparison = keys.iter()
                                .map(|key| format!("{}.{} = ?", alias, key))
                                .collect::<Vec<String>>()
                                .join(" AND ");

                            for entity in entities {
                                    if first {
                                        first = false;
                                    }else {
                                       delete_stmt.push_str(" OR ");
                                    }
                                   delete_stmt.push('(');
                                   //delete_stmt.push_str( &format!( #key_comparison, alias = alias));
                                   
                                delete_stmt.push_str(&key_comparison);

                                   delete_stmt.push(')');

                                  #(#key_params_code)*
                            }
                            if params.is_empty() {
                                return Ok(None);
                            }

                            Ok(Some((delete_stmt, params)))
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
