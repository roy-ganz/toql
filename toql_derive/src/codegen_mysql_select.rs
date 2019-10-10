
use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use syn::Ident;

use heck::SnakeCase;

pub(crate) struct GeneratedMysqlSelect<'a> {
    struct_ident: &'a Ident,
    sql_table_ident: Ident,
   

    select_columns: Vec<String>,
    select_columns_params: Vec<proc_macro2::TokenStream>,

    select_joins: Vec<String>,
    select_joins_params: Vec<proc_macro2::TokenStream>,

    select_keys: Vec<String>,
    
    select_key_types: Vec<proc_macro2::TokenStream>,
    select_key_fields: Vec<proc_macro2::TokenStream>,
    select_keys_params: Vec<proc_macro2::TokenStream>,

     merge_code: Vec<proc_macro2::TokenStream>
}


impl<'a> GeneratedMysqlSelect<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedMysqlSelect {
        let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        let sql_table_ident = Ident::new(
            &toql.table.clone().unwrap_or(renamed_table),
            Span::call_site(),
        );

        GeneratedMysqlSelect {
            struct_ident: &toql.ident,
            sql_table_ident: sql_table_ident,

          
         
            select_columns: Vec::new(),
            select_columns_params: Vec::new(),

            select_joins: Vec::new(),
            select_joins_params: Vec::new(),

            select_keys: Vec::new(),
          
            select_key_types: Vec::new(),
            select_key_fields: Vec::new(),
            select_keys_params: Vec::new(),

            

            merge_code: Vec::new()
        }
    }

    pub fn add_select_field(&mut self, toql: &Toql, field: &'a ToqlField)
    -> Result<(), ()>
    {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ident = field.ident.as_ref().unwrap();
        let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
        let sql_table_name = &self.sql_table_ident.to_string();
        let sql_table_alias = sql_table_name.to_snake_case();
        let field_type = field.first_non_generic_type().unwrap();

        // Regular field
        if field.join.is_empty() && field.merge. is_empty() {
            if field.key == true {

                let key_type = field.first_non_generic_type();
                self.select_key_types.push(quote!( #key_type));
               
               if field.number_of_options() > 0 {
                self.select_key_fields.push( quote!(self. #field_ident .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? ));
               } else {
                self.select_key_fields.push( quote!(self. #field_ident));
               }
               
                
                self.select_keys.push(format!("{}.{} = ?",sql_table_alias, sql_column));

                /*     // Normal key should may only one Option (Toql selectable)
                 self.select_keys_params.push( match field.number_of_options() {
                    1 => quote!( params.push( key
                                .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                .to_string()
                                ); ),
                    0 => quote!( params.push( key .to_string()); ),
                    _ => unreachable!()
                } 
                ); */
               
               
            } 
            self.select_columns.push(format!("{}.{}",sql_table_alias, sql_column));
        } 
        // Join field
        else if field.merge.is_empty() {

             /* Joins can also be fields.
                The key_type and key field on the joined struct must be provided
             */
            

             self.select_columns.push(String::from("true"));
             self.select_columns.push(String::from("{}"));
             self.select_columns_params.push( quote!(#field_type :: columns_sql()));
          
            let sql_join_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
            let default_join_alias = sql_join_table_name.to_snake_case();
            
          
            let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

            let mut on_condition: Vec<String>= Vec::new();
            for j in &field.join {

                if field.key == true {
                     if j.key_type.is_none() {
                       
                       self.select_key_types.push(quote_spanned! {
                            field_ident.span() =>
                            compile_error!("Key type is missing on joined struct: add `#[toql( join(key_type=\"..\"))]`")
                        });
                        return Err(());
                       
                       /*  return Err(quote_spanned! {
                            field_ident.span() =>
                            compile_error!("Key type is missing on joined struct: add `#[toql( join(key_type=\"..\"))]`")
                        }); */
                    } 
                    if j.key_field.is_none() {
                        self.select_key_types.push(quote_spanned! {
                            field_ident.span() =>
                            compile_error!("Key field is missing on joined struct: add `#[toql( join(key_field=\"..\"))]`")
                        });
                        return Err(());
                    }


                    let key_field = Ident::new(&j.key_field.as_ref().unwrap(), Span::call_site());

                    let key_type = Ident::new(&j.key_type.as_ref().unwrap(), Span::call_site());
                    self.select_key_types.push(quote!( #key_type));
                     let composite_key_field = format!("{}.{}", field_name, key_field);

                    if field.number_of_options() > 0 {
                       
                        self.select_key_fields.push( quote!(
                            Option::from(
                            self. #field_ident .as_ref() .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))?. #key_field)
                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(#composite_key_field)))?
                            ));
                    } else {
                        self.select_key_fields.push( quote!( Option::from(self. #field_ident . #key_field)
                                    .ok_or(toql::error::ToqlError::ValueMissing( String::from(#composite_key_field)))?
                        ));
                    }
                
                    self.select_keys.push(format!("{}.{} = ?",sql_table_alias, j.this_column.as_ref().unwrap()));

                    /*     // Normal key should may only one Option (Toql selectable)
                    self.select_keys_params.push( match field.number_of_options() {
                        1 => quote!( params.push( key
                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                    .to_string()
                                    ); ),
                        0 => quote!( params.push( key .to_string()); ),
                        _ => unreachable!()
                    } 
                    ); */
                             
             } 


                let auto_self_key = crate::util::rename(&format!("{}_id", field_name), &toql.columns);
                let self_column = j.this_column.as_ref().unwrap_or(&auto_self_key);


                let default_other_column = crate::util::rename("id", &toql.columns);
                /* let other_field =
                    Ident::new(&j.other_column.as_ref().unwrap_or(&default_other_column), Span::call_site()); */

                let other_column =j.other_column.as_ref().unwrap_or(&default_other_column);
                on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias,other_column, ));

                // TODO custom on clause
            }


          
            // TODO rename join entity

             self.select_joins.push(format!("JOIN {} {} ON ({}) {{}}",sql_join_table_name, field_name, on_condition.join(" AND ")  ));
             self.select_joins_params.push( quote!(#field_type :: joins_sql()));
        } 
        // Merge field
        else {
           
            let sql_join_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
             let default_join_alias = sql_join_table_name.to_snake_case();
            
          
            let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

             let mut on_condition: Vec<String>= Vec::new();
            for j in &field.merge {
                //let auto_self_key : String = crate::util::rename(&field_ident.to_string(), &toql.columns);
                let self_column :String = crate::util::rename(&j.this_field.to_string(), &toql.columns);

                let other_column = crate::util::rename(&j.other_field.to_string(), &toql.columns);
                on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias,other_column, ));
            }


            let merge_field_init = if field.number_of_options() > 0 {
                 quote!( Some(Vec::new())) 
            } else {
                quote!(Vec::new()) 
            };
            
           
            let dep_join = format!("JOIN {} {} ON ({} AND {{}})", sql_table_name, sql_table_alias, on_condition.join(" AND "));

            let struct_ident = self.struct_ident;
            let merge_function = Ident::new( &format!("merge_{}", &field.ident.as_ref().unwrap()),  Span::call_site());

         
            self.merge_code.push(quote!(
                    let #field_ident = #field_type :: select_dependencies( &format!(#dep_join, key_predicate), &params, conn)?;
                     for e in entities.iter_mut() { e . #field_ident = #merge_field_init; }
                    #struct_ident :: #merge_function(&mut entities, #field_ident); 
            )); 

        }
        Ok(())
    }
  
}


impl<'a> quote::ToTokens for GeneratedMysqlSelect<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

         let sql_table_name = &self.sql_table_ident.to_string();
            let sql_table_alias = sql_table_name.to_snake_case(); // TODO rename

      

        let key_types = &self.select_key_types;
      

        let key_type_code = if key_types.len() == 1 { quote!( #(#key_types)* ) }  else { quote!( ( #(#key_types),* ))};

         //  let key_params_code = &self.key_params_code;

        let mods = if self.select_keys.is_empty() {
            quote!( /* Skipped code generation, because #[toql(key)] is missing */ )
        /*  quote_spanned! {
            struct_ident.span() =>
            compile_error!( "cannot find key(s) to delete and update: add `#[toql(key)]` to at the field(s) in your struct that are the primary key in your table");
        } */
        } else {
           
          
           let select_columns = self.select_columns.join(",");
           
           let select_columns_params= &self.select_columns_params;
           let select_joins = self.select_joins.join(" ");
              
            let select_joins_params=  &self.select_joins_params;
            
            let select_keys_params= &self.select_keys_params;

            let select_statement= format!("SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name,  sql_table_alias);

            let select_stmt= format!("SELECT {{}} FROM {} {} {{}}WHERE {}",
                sql_table_name,  sql_table_alias, self.select_keys.join(" AND "));
            let select_one_stmt = format!("{} LIMIT 0,2", select_stmt);

               let select_dependend_stmt= format!("SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name,  sql_table_alias);

            let merge_code = &self.merge_code;
            let merge_key_predicate = self.select_keys.join(" AND ");
          

        let select_keys_params : Vec<proc_macro2::TokenStream> = if key_types.len() == 1 { 
                            vec![quote!( params.push(key.to_string()); )]
                            } else {  
                                self.select_key_types.iter().enumerate().map(|x| { 
                                    let i = x.0;  
                                    let is = syn::Index::from(i);
                                    quote!(params.push(key. #is .to_string()); )} ).collect()
                            };
            let columns_sql_code = if select_columns_params.is_empty() {
                quote!( String::from(#select_columns)) 
                } else {
                    quote!(format!(#select_columns, #(#select_columns_params),*))
                };
            let joins_sql_code = if select_columns_params.is_empty() {
                quote!( String::from(#select_joins)) 
                } else {
                    quote!(format!(#select_joins, #(#select_joins_params),*))
                };

        let select_key_fields =  &self.select_key_fields;

           let key_getter = if  select_key_fields.len() == 1 {
               quote!( #(#select_key_fields .to_owned())* )
           } else {
                quote!( ( #(#select_key_fields .to_owned()),* ))
           };
        


            quote! {

                impl toql::key::Key<#struct_ident> for #struct_ident {
                    type Key = #key_type_code;

                    fn key(&self) -> Result<Self::Key, toql::error::ToqlError> {
                       Ok( #key_getter )
                    }
                }
                

                impl<'a> toql::mysql::select::Select<#struct_ident> for #struct_ident {

                 
                    fn columns_sql() -> String {
                           #columns_sql_code

                    }
                    fn joins_sql() -> String {
                            #joins_sql_code
                    }
                    fn select_sql(join: Option<&str>) -> String {
                            format!( #select_statement, 
                            Self::columns_sql(), Self::joins_sql(),join.unwrap_or(""))
                    }


                     fn select_one(key: &<#struct_ident as toql::key::Key<#struct_ident>>::Key, conn: &mut toql::mysql::mysql::Conn) 
                     -> Result<#struct_ident,  toql::error::ToqlError>
                     {
                        let select_stmt = format!( "{} WHERE {} LIMIT 0,2", Self::select_sql(None), #merge_key_predicate);

                        let mut params :Vec<String> = Vec::new();
                       
                        #(#select_keys_params)*
                        toql::log_sql!(select_stmt, params);
                        
                        let entities_stmt = conn.prep_exec(select_stmt, &params)?;
                        let mut entities = toql::mysql::row::from_query_result::< #struct_ident>(entities_stmt)?;

                        if entities.len() > 1 {
                            return Err( toql::error::ToqlError::NotUnique);
                        } else if entities.is_empty() {
                            return Err( toql::error::ToqlError::NotFound);
                        }

                        let key_predicate = #merge_key_predicate;
                        #(#merge_code)*
                        Ok(entities.pop().unwrap())
                     }

                       
                        fn select_many(
                            key: &<#struct_ident as toql::key::Key<#struct_ident>>::Key,
                            conn: &mut toql::mysql::mysql::Conn,
                            first: u64,
                            max: u16
                        ) -> Result<Vec< #struct_ident> ,  toql::error::ToqlError>{
                                unimplemented!();


                        }

                        fn select_dependencies(join: &str, params:&Vec<String>,
                            conn: &mut toql::mysql::mysql::Conn) -> Result<Vec<#struct_ident> ,  toql::error::ToqlError>{
                                let select_stmt =  Self::select_sql(Some(join));
                                //let select_stmt = format!( #select_dependend_stmt, Self::columns_sql(), Self::joins_sql(), join);

                        
                        
                        
                        toql::log_sql!(select_stmt, params);
                        
                        let entities_stmt = conn.prep_exec(select_stmt, params)?;
                        let mut entities = toql::mysql::row::from_query_result::<#struct_ident>(entities_stmt)?;

                        

                        let key_predicate = #merge_key_predicate;
                        #(#merge_code)*
                        
                        Ok(entities)
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