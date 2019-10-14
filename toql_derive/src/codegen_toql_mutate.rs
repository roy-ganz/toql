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

    insert_columns: Vec<String>,
    insert_params_code: Vec<proc_macro2::TokenStream>,

    keys: Vec<String>,
    key_params_code: Vec<proc_macro2::TokenStream>,

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
            insert_columns: Vec::new(),
            insert_params_code: Vec::new(),

            keys: Vec::new(),
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
        if field.join.is_empty() {
            let field_ident = field.ident.as_ref().unwrap();
            let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
            self.insert_columns.push(sql_column);
           
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

        else {
            let field_ident = field.ident.as_ref().unwrap();
            for j in &field.join {
                let auto_self_key = crate::util::rename(&format!("{}_id", field_name), &toql.columns);
                let self_column = j.this_column.as_ref().unwrap_or(&auto_self_key);
                self.insert_columns.push(self_column.to_owned());
                let default_other_column = crate::util::rename("id", &toql.columns);
                let other_field =
                    Ident::new(&j.other_column.as_ref().unwrap_or(&default_other_column), Span::call_site());

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
                                // TODO Option wrapping
                                    quote!(
                                            params.push(entity. #field_ident
                                                .as_ref(). map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                // TODO object tolerance on foreign field
                                let composite_field_name = format!("{}.{}",&field_name , &other_field );
                                quote!(
                                                params.push(
                                                   entity. #field_ident
                                                    .as_ref()
                                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                                    . #other_field
                                                     .to_string());
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

        // SQL code cannot be updated, skip field
        if field.sql.is_some() {
            return;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field.ident.as_ref().unwrap().to_string();
        let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

        // Key field
        if field.key {
            if field.join.is_empty() {
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
            self.keys
                .push(field.ident.as_ref().unwrap().to_string());
            } 
            // Join used as field
            else {
                // Quick and dirty solution
                 // Option<key> (Toql selectable)

                for j in &field.join {

                    let auto_key_field= String::from("id");
                    let key_field = Ident::new(j.key_field.as_ref().unwrap_or(&auto_key_field), Span::call_site()); // TODO compiler error

                    // Keys for insert and delete may never be null
                    if field._first_type() == "Option" {
                        self.key_params_code.push( quote!(
                            params.push(entity. #field_ident. as_ref()
                            .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?. #key_field. to_string().to_owned() );
                        ));
                    } else {
                        self.key_params_code
                            .push(quote!(params.push(entity. #field_ident. #key_field. to_string().to_owned()); ));
                    }

                    // Add field to keys, struct may contain multiple keys (composite key) 

                    // auto 
                   let auto_self_column = crate::util::rename(&format!("{}_id", field_name), &toql.columns);
                 let self_column = j.this_column.as_ref().unwrap_or(&auto_self_column);

                    /* self.keys
                        .push(j.this_column.as_ref().unwrap_or().to_string());
                    } */ 
                    self.keys
                        .push(self_column.to_string());
                    } 
                


            }
        }

        // Field is not skipped for update
         if !field.skip_mut {
            // Regular field
            if field.join.is_empty() && field.merge.is_empty() {
                   
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
            else if field.merge.is_empty(){

                for j in &field.join {
                    let auto_self_key =
                        crate::util::rename(&format!("{}_id", field_name), &toql.columns);
                    let self_column = j.this_column.as_ref().unwrap_or(&auto_self_key);
                    let default_other_column = crate::util::rename("id", &toql.columns);
                
                    let other_field =
                        Ident::new(&j.other_column.as_ref().unwrap_or(&default_other_column).to_snake_case(), Span::call_site());
                    let set_statement = format!("{{}}.{} = ?, ", &self_column);

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
                                                    params.push(
                                                        entity. #field_ident
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
                    // diff code
                    self.diff_set_code.push(
                            match field.number_of_options()  {
                                2 => { // Option<Option<T>>
                                    quote!(
                                        if entity. #field_ident .is_some() 
                                        &&  outdated. #field_ident .as_ref() 
                                        .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?  
                                        != entity .  #field_ident .as_ref() .unwrap()
                                        {
                                            
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
                                            if outdated. #field_ident .as_ref() != entity. #field_ident.as_ref() {
                                                update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(entity. #field_ident
                                                    .as_ref(). map_or(String::from("NULL"), |e| e. #other_field .to_string()));
                                            }
                                    )
                                },

                                1 if !field.preselect => { // Option<T>
                                    quote!(
                                            if entity. #field_ident .is_some() 
                                             &&  outdated. #field_ident .as_ref()
                                             .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?  
                                             .  #other_field
                                             != entity .  #field_ident .as_ref().unwrap() .  #other_field
                                            {
                                                update_stmt.push_str( &format!(#set_statement, alias));
                                                params.push(entity. #field_ident
                                                    .as_ref().unwrap(). #other_field .to_string());
                                            }
                                    )
                                },
                                _ => { // T
                                    quote!(
                                         if outdated. #field_ident .as_ref() != entity. #field_ident.as_ref() {
                                            update_stmt.push_str( &format!(#set_statement, alias));
                                            params.push(entity. #field_ident . #other_field .to_string());
                                         }
                                    )
                                }
                            }
                    );
                }
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

        let key_comparison = self
            .keys
            .iter()
            .map(|k| format!("{{alias}}.{} = ?", k))
            .collect::<Vec<String>>()
            .join(" AND ");

        let update_set_code = &self.update_set_code;
         let diff_set_code = &self.diff_set_code;

        let insert_params_code = &self.insert_params_code;
        let key_params_code = &self.key_params_code;

        let diff_merge_code = &self.diff_merge_code;

        let mods = if self.keys.is_empty() {
            quote!( /* Skipped code generation, because #[toql(key)] is missing */ )
        /*  quote_spanned! {
            struct_ident.span() =>
            compile_error!( "cannot find key(s) to delete and update: add `#[toql(key)]` to at the field(s) in your struct that are the primary key in your table");
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
                    .join(", ")
            );
            let insert_statement = format!(
                "INSERT INTO {} ({}) VALUES",
                self.sql_table_ident,
                self.insert_columns.join(",")
            );

            /*  let update_where_statement = format!(" WHERE {}", key_comparison);
            let delete_one_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE {}",
                self.sql_table_ident, key_comparison
            ); */
            let delete_many_statement = format!(
                "DELETE {{alias}} FROM {} {{alias}} WHERE ",
                self.sql_table_ident
            );

             
            quote! {

               
                

                impl<'a> toql::mutate::Mutate<'a, #struct_ident> for #struct_ident {



                     fn insert_many_sql<I>(entities: I)-> toql::error::Result<Option<(String, Vec<String>)>>
                     where I: IntoIterator<Item=&'a #struct_ident> + 'a
                     {


                            let mut params :Vec<String>= Vec::new();
                            let mut insert_stmt = String::from( #insert_statement);

                            for entity in entities {
                                insert_stmt.push_str( #insert_cols );
                                #(#insert_params_code)*
                            }
                             if params.is_empty() {
                                return Ok(None);
                            }
                            Ok(Some((insert_stmt, params)))
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

                        #(#key_params_code)*

                        Ok((update_stmt, params))

                    } */
                    fn update_many_sql<I>(entities:I) -> toql::error::Result<Option<(String, Vec<String>)>>
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
                            update_stmt.push_str( &format!(#key_comparison, alias = alias));

                            #(#key_params_code)*
                         }

                        Ok(Some((update_stmt, params)))

                    }
                    fn shallow_diff_many_sql<I>(entities:I) -> toql::error::Result<Option<(String, Vec<String>)>>
                    where I: IntoIterator<Item=(&'a #struct_ident, &'a #struct_ident)> + 'a + Clone
                    {
                        let mut params: Vec<String> = Vec::new();
                        let mut update_stmt = String::from("UPDATE ");
                        let mut first = true;

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
                            update_stmt.push_str( &format!(#key_comparison, alias = alias));

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
                            let mut first = true;
                            for entity in entities {
                                    if first {
                                        first = false;
                                    }else {
                                       delete_stmt.push_str(" OR ");
                                    }
                                   delete_stmt.push('(');
                                   delete_stmt.push_str( &format!( #key_comparison, alias = alias));
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
