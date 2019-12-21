/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;
use syn::Ident;

pub(crate) struct GeneratedToqlDelup<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
   
    key_params_code: Vec<TokenStream>,
    key_columns_code: Vec<TokenStream>,

    update_set_code: Vec<TokenStream>,
    diff_set_code: Vec<TokenStream>,

    diff_merge_code: Vec<TokenStream>,
}

impl<'a> GeneratedToqlDelup<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlDelup {
        GeneratedToqlDelup {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
           
            key_columns_code: Vec::new(),
            key_params_code: Vec::new(),

            update_set_code: Vec::new(),
            diff_set_code: Vec::new(),

            diff_merge_code: Vec::new(),
        }
    }
    
    pub(crate) fn add_delup_field(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_type_ident = &field.rust_type_ident;

        // Handle key predicate and parameters
        match &field.kind {
            FieldKind::Regular(regular_attrs) => {
                // SQL code cannot be updated, skip field
                if let SqlTarget::Expression(_) = regular_attrs.sql_target {
                    return;
                };

                if regular_attrs.key {
                    if field.number_of_options > 0 {
                        self.key_params_code.push( quote!(
                        params.push(entity. #rust_field_ident. as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?.to_string().to_owned() );
                    ));
                    } else {
                        self.key_params_code.push(
                            quote!(params.push(entity. #rust_field_ident.to_string().to_owned()); ),
                        );
                    }

                    // Add field to keys, struct may contain multiple keys (composite key)
                    self.key_columns_code.push(quote!(
                        keys.push(String::from(#rust_field_name));
                    ));
                }
            }
            FieldKind::Join(join_attrs) => {
                if join_attrs.key {
                    let unwrap = match field.number_of_options {
                        1 if !field.preselect => quote!(.as_ref().unwrap()),
                        0 => quote!(),
                        _ => {
                            quote!() /*TODO throw error, invalid key*/
                        }
                    };

                    self.key_params_code
                                .push(
                                        quote!(
                                            params.extend_from_slice( &<#rust_type_ident as toql::key::Key>::params(  &<#rust_type_ident as toql::key::Key>::get_key( &entity . #rust_field_ident #unwrap )?));
                                        )
                                    );
                }
            }
            FieldKind::Merge(_) => {}
        };

        
        if field.skip_mut {
            return;
        }
        

        match &field.kind {
            FieldKind::Regular(regular_attrs) => {
                let set_statement = format!(
                    "{{}}.{} = ?, ",
                    match &regular_attrs.sql_target {
                        SqlTarget::Column(ref sql_column) => sql_column,
                        _ => {
                            return;
                        }
                    }
                );

                // Option<T>, <Option<Option<T>>
                if field.number_of_options > 0 && !field.preselect {
                    let unwrap_null = if 2 == field.number_of_options {
                        quote!(.as_ref().map_or(String::from("NULL"), |x| x.to_string()))
                    } else {
                        quote!()
                    };

                    // update statement
                    // Doesn't update primary key
                    if !regular_attrs.key {
                        self.update_set_code.push(quote!(
                            if entity. #rust_field_ident .is_some() {
                                update_stmt.push_str( &format!(#set_statement, alias));
                                params.push(entity . #rust_field_ident .as_ref().unwrap() #unwrap_null .to_string() .to_owned());
                            }
                            ));
                    }
                    // diff statement
                    self.diff_set_code.push(quote!(
                        if entity. #rust_field_ident .is_some()
                         && outdated. #rust_field_ident  .as_ref()
                            .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))? != entity. #rust_field_ident .as_ref().unwrap()
                         {
                                update_stmt.push_str( &format!(#set_statement, alias));
                                params.push(entity . #rust_field_ident .as_ref().unwrap() #unwrap_null .to_string() .to_owned());
                        }
                    ));
                }
                // T, Option<T> (nullable column)
                else {
                    let unwrap_null = if 1 == field.number_of_options {
                        quote!(.map_or(String::from("NULL"), |x| x.to_string()))
                    } else {
                        quote!()
                    };
                    //update statement
                    if !regular_attrs.key {
                        self.update_set_code.push(quote!(
                        update_stmt.push_str( &format!(#set_statement, alias));
                        params.push( entity . #rust_field_ident #unwrap_null .to_string() .to_owned());
                            ));
                    }

                    // diff statement
                    self.diff_set_code.push(quote!(
                        if outdated.  #rust_field_ident != entity. #rust_field_ident
                        {
                                update_stmt.push_str( &format!(#set_statement, alias));
                                 params.push( entity . #rust_field_ident #unwrap_null .to_string() .to_owned());

                        }
                    ));
                }
            }
            FieldKind::Join(join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;

                let add_columns_to_update_stmt = quote!(
                     for other_column in <#rust_type_ident as toql::key::Key>::columns() {
                        #default_self_column_code;
                        let self_column = #columns_map_code;
                        update_stmt.push_str(&format!("{}.{} = ?, ",alias, self_column));
                    }
                );

                let set_params_code = match field.number_of_options {
                    2 => {
                        // Option<Option<T>>
                        quote!( params.extend_from_slice(
                                   &entity. #rust_field_ident
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(|  |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                    .map(|c| String::from("NULL")).collect::<Vec<_>>())},
                                        | some| { Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(some)?)) })?

                                    );
                        )
                    }
                    1 if field.preselect => {
                        // #[toql(preselect)] Option<T>
                        quote!(
                            params.extend_from_slice(
                                   &entity
                                    . #rust_field_ident .as_ref()
                                   .map_or_else::<Result<Vec<String>,toql::error::ToqlError>,_,_>(|  |{ Ok(<#rust_type_ident as toql::key::Key>::columns().iter()
                                    .map(|c| String::from("NULL")).collect::<Vec<_>>())},
                                   | some| {
                                       Ok(<#rust_type_ident as toql::key::Key>::params( &<#rust_type_ident as toql::key::Key>::get_key(some)?))
                                       })?);
                        )
                    }
                    1 if !field.preselect => {
                        // Option<T>
                        quote!(
                             params.extend_from_slice( &<#rust_type_ident as toql::key::Key>::params(
                                        &<#rust_type_ident as toql::key::Key>::get_key(entity. #rust_field_ident .as_ref().unwrap())?));
                        )
                    }
                    _ => {
                        // T
                        quote!(
                                           params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params(
                                               &<#rust_type_ident as toql::key::Key>::get_key(&entity. #rust_field_ident)?));


                        )
                    }
                };

                self.update_set_code.push(match field.number_of_options {
                    2 => {
                        // Option<Option<T>>
                        quote!(
                            if entity. #rust_field_ident .is_some() {
                               #add_columns_to_update_stmt
                               #set_params_code
                            }
                        )
                    }
                    1 if field.preselect => {
                        // #[toql(preselect)] Option<T>
                        quote!(
                                 #add_columns_to_update_stmt
                            #set_params_code
                        )
                    }

                    1 if !field.preselect => {
                        // Option<T>
                        quote!(
                                if entity. #rust_field_ident .is_some() {
                                      #add_columns_to_update_stmt
                                      #set_params_code
                                }
                        )
                    }
                    _ => {
                        // T
                        quote!(
                              #add_columns_to_update_stmt
                              #set_params_code
                        )
                    }
                });
                // diff code
                //let join_key_index = syn::Index::from(self.key_params_code.len() - 1);
                self.diff_set_code.push(
                            match field.number_of_options  {
                                2 => { // Option<Option<T>>
                                    quote!(
                                        if entity. #rust_field_ident .is_some()
                                        &&
                                         entity. #rust_field_ident
                                                    .as_ref() .unwrap()
                                                    .as_ref()
                                                    .map_or::<Result<_,toql::error::ToqlError>,_>(Ok(None), |e| {
                                                                Ok(Some(toql::key::Key::get_key(e)?))
                                                    })?
                                        !=  outdated. #rust_field_ident
                                        .as_ref() .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?
                                        .as_ref().map_or::<Result<_,toql::error::ToqlError>,_>(Ok(None), |e| {
                                                            Ok(Some(toql::key::Key::get_key(e)? ))
                                         })?
                                        {
                                            #add_columns_to_update_stmt
                                            #set_params_code
                                        }
                                    )
                                    },
                                1 if field.preselect => { // #[toql(preselect)] Option<T>
                                    quote!(
                                            if    entity. #rust_field_ident
                                                    .as_ref()
                                                    .map_or::<Result<_,toql::error::ToqlError>,_>(Ok(None), |e| {
                                                                Ok(Some(toql::key::Key::get_key(e)?))
                                                    })?
                                                !=  outdated. #rust_field_ident
                                                    .as_ref()
                                                    .map_or::<Result<_,toql::error::ToqlError>,_>(Ok(None), |e| {
                                                                Ok(Some(toql::key::Key::get_key(e)? ))
                                                })?
                                            {
                                            #add_columns_to_update_stmt
                                            #set_params_code
                                            }
                                    )
                                },
                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                            if entity. #rust_field_ident .is_some()
                                            && toql::key::Key::get_key(entity .  #rust_field_ident.as_ref() .unwrap())?
                                             !=  toql::key::Key::get_key(outdated .  #rust_field_ident.as_ref()
                                             .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?
                                              )?
                                            {
                                               #add_columns_to_update_stmt
                                               #set_params_code
                                            }
                                    )
                                },
                                _ => { // T

                                    quote!(
                                         if toql::key::Key::get_key(&outdated. #rust_field_ident)?
                                            !=  toql::key::Key::get_key(&entity. #rust_field_ident)? {
                                            #add_columns_to_update_stmt
                                            #set_params_code
                                         }
                                    )
                                }
                            }
                    );
            }
            FieldKind::Merge(_) => {
                let optional_unwrap = if field.number_of_options > 0 {
                    quote!( .unwrap())
                } else {
                    quote!()
                };
                let optional_if = if field.number_of_options > 0 {
                    quote!(if entity .  #rust_field_ident  .is_some() )
                } else {
                    quote!()
                };
                let optional_ok_or = if field.number_of_options > 0 {
                    quote!(  .ok_or(toql::error::ToqlError::ValueMissing(String::from(#rust_field_name)))?)
                } else {
                    quote!()
                };

                self.diff_merge_code.push( quote!(
                        for (boutdated, bentity) in entities {
                             let outdated = boutdated.borrow();
                             let entity = bentity.borrow();
                            #optional_if {
                                 let (insert_sql, diff_sql, delete_sql) = toql::mutate::collection_delta_sql::<#rust_type_ident,Self, Self, toql::dialect::Generic>(
                                     outdated. #rust_field_ident .as_ref() #optional_ok_or,
                                    entity.#rust_field_ident .as_ref() #optional_unwrap )?;

                                  if let Some( s) = insert_sql {
                                        sql.push(s);
                                    }
                                    if let Some( s) = diff_sql {
                                        sql.push(s);
                                    }
                                    if let Some( s) = delete_sql {
                                        sql.push(s);
                                    }
                                }
                        }
                ));
            }
        }
    }

   
}
impl<'a> quote::ToTokens for GeneratedToqlDelup<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let update_set_code = &self.update_set_code;
        let diff_set_code = &self.diff_set_code;

       
        let key_params_code = &self.key_params_code;

        let diff_merge_code = &self.diff_merge_code;

        // Generate modules if there are keys available
        let mods = {
           

            let delete_many_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE ",
                self.sql_table_name
            );

            let key_columns_code = &self.key_columns_code;
            let sql_table_name = &self.sql_table_name;

            quote! {

                impl<'a> toql::mutate::Delete<'a, #struct_ident> for toql::dialect::Generic {
                    fn delete_many_sql(keys: &[<#struct_ident as toql::key::Key>::Key]) -> toql::error::Result<Option<(String, Vec<String>)>>
                        {
                            let alias= "t";
                            let mut delete_stmt =format!(#delete_many_statement, alias = alias);

                            let mut params :Vec<String>= Vec::new();
                            
                            let mut first = true;

                           

                              let key_comparison = <#struct_ident as toql::key::Key>::columns().iter()
                                .map(|key| format!("{}.{} = ?", alias, key))
                                .collect::<Vec<String>>()
                                .join(" AND ");

                            for key in keys {
                                    if first {
                                        first = false;
                                    }else {
                                       delete_stmt.push_str(" OR ");
                                    }
                                   delete_stmt.push('(');

                                delete_stmt.push_str(&key_comparison);

                                   delete_stmt.push(')');
                                   params.extend_from_slice(&<#struct_ident as toql::key::Key>::params(&key));
                            }
                            if params.is_empty() {
                                return Ok(None);
                            }

                            Ok(Some((delete_stmt, params)))
                     }


                }
                impl<'a> toql::mutate::Update<'a, #struct_ident> for toql::dialect::Generic {


                    fn update_many_sql<Q : std::borrow::Borrow<#struct_ident>>(entities: &[Q]) -> toql::error::Result<Option<(String, Vec<String>)>>
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;
                        let mut keys: Vec<String> = Vec::new();

                         #(#key_columns_code)*



                        // Generate  join
                        for (i, bentity) in entities.iter().enumerate() {
                            let entity = bentity.borrow();

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
                         for (i, bentity) in entities.iter().enumerate() {
                             let entity = bentity.borrow();
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
                         for (i, bentity) in entities.iter().enumerate() {
                             let entity = bentity.borrow();
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
                }
                impl<'a, T: toql::mysql::mysql::prelude::GenericConnection + 'a> toql::mutate::Diff<'_,#struct_ident>  for toql::mysql::MySql<'a,T>
                {
                    fn shallow_diff_many_sql<Q : std::borrow::Borrow<#struct_ident>>(entities: &[(Q, Q)]) -> toql::error::Result<Option<(String, Vec<String>)>>
                    
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut keys: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;

                        #(#key_columns_code)*


                        // Generate  join
                        for (i, (boutdated, bentity)) in entities.iter().enumerate() {
                            let outdated = boutdated.borrow();
                            let entity = bentity.borrow();

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
                         for (i, (boutdated, bentity)) in entities.iter().enumerate() {
                                let outdated = boutdated.borrow();
                                let entity = bentity.borrow();
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
                         for (i, (boutdated, bentity)) in entities.iter().enumerate() {
                            let outdated = boutdated.borrow();
                            let entity = bentity.borrow();

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
                    fn diff_many_sql<Q : std::borrow::Borrow<#struct_ident>>(entities: &[(Q, Q)]) -> toql::error::Result<Option<Vec<(String,Vec<String>)>>>
                    {

                        let mut sql: Vec<(String, Vec<String>)> = Vec::new();

                        let update = <Self as  toql::mutate::Diff<#struct_ident>>::shallow_diff_many_sql(entities)?;
                        if update.is_some() {
                            sql.push(update.unwrap());
                        }

                            #(#diff_merge_code)*

                            if sql.is_empty() {
                                return Ok(None);
                            }

                            Ok(Some(sql))

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
