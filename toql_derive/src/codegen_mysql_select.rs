
use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use syn::Ident;

use heck::SnakeCase;
use heck::MixedCase;

pub(crate) struct GeneratedMysqlSelect<'a> {
    struct_ident: &'a Ident,
    sql_table_ident: Ident,
    vis: &'a syn::Visibility,   

    select_columns: Vec<String>,
    select_columns_params: Vec<proc_macro2::TokenStream>,

    select_joins: Vec<String>,
    select_joins_params: Vec<proc_macro2::TokenStream>,

    select_keys: Vec<String>,
    
    select_key_types: Vec<proc_macro2::TokenStream>,
    select_key_fields: Vec<proc_macro2::TokenStream>,
    select_keys_params: Vec<proc_macro2::TokenStream>,
    key_predicates: Vec<proc_macro2::TokenStream>,

    key_setters: Vec<proc_macro2::TokenStream>,

    merge_code: Vec<proc_macro2::TokenStream>,
    key_columns_code: Vec<proc_macro2::TokenStream>,
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
            vis : &toql.vis,
          
         
            select_columns: Vec::new(),
            select_columns_params: Vec::new(),

            select_joins: Vec::new(),
            select_joins_params: Vec::new(),

            select_keys: Vec::new(),
          
            select_key_types: Vec::new(),
            select_key_fields: Vec::new(),
            select_keys_params: Vec::new(),
            key_predicates: Vec::new(),
            key_setters: Vec::new(),


            

            merge_code: Vec::new(),
            key_columns_code: Vec::new()
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
        if field.join.is_none() && field.merge. is_empty() {
            if field.key == true {

                
                 self.key_columns_code.push( quote!( columns.push( String::from(#sql_column)); ));
                let key_type = field.first_non_generic_type();
                self.select_key_types.push(quote!( #key_type));
               
               if field.number_of_options() > 0 {
                   let value= quote!(self. #field_ident .as_ref() .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? .to_owned());
                    self.select_key_fields.push( value);
                
                let index =  syn::Index::from(self.select_key_types.len()-1);
                self.key_setters.push( quote!(self. #field_ident = Some( key . #index  ); ))
               } else {

                self.select_key_fields.push( quote!(self. #field_ident .to_owned()));

                 let index =  syn::Index::from(self.select_key_types.len()-1);
                 
                 self.key_setters.push( quote!(self. #field_ident = key . #index;) )
               }
               
                
                self.select_keys.push(format!("{}.{} = ?",sql_table_alias, sql_column));

              let toql_field = field_name.to_mixed_case();
              let key_index= syn::Index::from(self.select_key_fields.len() - 1);
              self.key_predicates.push(  quote! {
                 .and(toql::query::Field::from(#toql_field).eq( key . #key_index))
                });
              self.select_keys_params.push(  quote! {
                 params.push( key . #key_index .to_string());
                });
              

               

                     // Normal key should may only one Option (Toql selectable)
                 /* self.select_keys_params.push( match field.number_of_options() {
                    1 => quote!( params.push( key
                                .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                .to_string()
                                ); ),
                    0 => quote!( params.push( key .to_string()); ),
                    _ => unreachable!()
                } 
                );  */
               
               
            } 
            self.select_columns.push(format!("{}.{}",sql_table_alias, sql_column));
        } 
        // Join field
        else if field.join.is_some() {

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


            let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_name), &toql.columns)];
            let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() { 
                field.join.as_ref().unwrap().this_columns.as_ref() }
                else {
                    &default_self_columns
                };
                self.key_columns_code.push( quote!( columns.extend_from_slice(&<#field_type as toql::key::Key>::columns());));

                let default_other_columns= vec![crate::util::rename("id", &toql.columns)];
            let other_columns =  if !field.join.as_ref().unwrap().other_columns.is_empty() { 
                field.join.as_ref().unwrap().other_columns.as_ref() }
                else {
                    &default_other_columns
                };
            self_columns.iter().zip(other_columns).enumerate().for_each( |(i, (self_column, other_column))| {

                if field.key == true {
                 
                    // Join always on key 
                                     
                    let struct_key_type = Ident::new(&format!("{}Key", &field_type), Span::call_site());
                    self.select_key_types.push(quote!( <#field_type as toql::key::Key>::Key));
                  

                       let toql_field = format!("{}_{}",field_name.to_mixed_case(), other_column.to_string().to_mixed_case());
                       let key_index= syn::Index::from(self.select_key_types.len() -1); 
                       let join_key_index= syn::Index::from(i); 
                        self.key_predicates.push(  quote! {
                        .and(toql::query::Field::from(#toql_field).eq ((key . #key_index) .#join_key_index ))
                        });

                    if field.number_of_options() > 0 {
                        self.select_key_fields.push( quote!(
                            < #field_type as toql::key::Key>::get_key( 
                                self. #field_ident .as_ref()
                                    .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))?
                                )?
                        ));

                        let index =  syn::Index::from(self.select_key_types.len()-1);
                          self.key_setters.push( quote!(
                            < #field_type as toql::key::Key>::set_key(self. #field_ident .as_mut()
                                                    .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? , key . #index );
                        ));
                       
                    } else {
                         self.select_key_fields.push( quote!(
                            < #field_type as toql::key::Key>::get_key(  &self. #field_ident )?
                        ));

                       // let index =  syn::Index::from(self.select_key_types.len()-1);
                          self.key_setters.push( quote!(
                            < #field_type as toql::key::Key>::set_key(&mut self. #field_ident,key . #key_index);
                        ));

                         self.select_keys.push(format!("{}.{} = ?", sql_table_alias,self_column ));
                         self.select_keys_params.push(  quote! {
                            params.push( (key . #key_index) . #join_key_index .to_string());
                            });
                      
                    }

                   
                             
             } 



                on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias, other_column ));

                // TODO custom on clause
            }); 


          
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
                let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
                let auto_self_field= "id".to_string();

                let self_column :String = crate::util::rename(&j.this_field.as_ref().unwrap_or(&auto_self_field).to_string(), &toql.columns);

                let other_column = crate::util::rename(&j.other_field.as_ref().unwrap_or(&auto_other_field).to_string(), &toql.columns);
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
      

        let key_type_code =  quote!(  #(pub #key_types),* );

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
            
           // let select_keys_params= &self.select_keys_params;

            let select_statement= format!("SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name,  sql_table_alias);

            let select_stmt= format!("SELECT {{}} FROM {} {} {{}}WHERE {}",
                sql_table_name,  sql_table_alias, self.select_keys.join(" AND "));
            let select_one_stmt = format!("{} LIMIT 0,2", select_stmt);

               let select_dependend_stmt= format!("SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name,  sql_table_alias);

            let merge_code = &self.merge_code;
            let merge_key_predicate = self.select_keys.join(" AND ");
          

        let select_keys_params = &self.select_keys_params; /* Vec<proc_macro2::TokenStream> = self.select_key_types.iter().enumerate().map(|x| { 
                                    let i = x.0;  
                                    let is = syn::Index::from(i);
                                    quote!(params.push(key. #is .to_string()); )} ).collect(); */

                            
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
       // let key_setters_fragments =  &self.key_setters;

            let key_getter =  quote!( #(#select_key_fields  ),* );
            
           
/* 
           let key_setters = if  key_setters_fragments.len() == 1 {
               vec![quote!( #(#key_setters_fragments ;)* )];
           } else {
               key_setters_fragments.iter().enumerate().map(|x| { 
                                    let s = x.1;
                                    let i = x.0;  
                                    let is = syn::Index::from(i);
                                    quote!(#s . #is; )} ).collect();
                 
           };  */
        

            let key_predicates = &self.key_predicates;
                /* self.key_predicates.iter().enumerate().map(|x| { 
                                    let s = x.1;
                                    let i = x.0;  
                                    let is = syn::Index::from(i);
                                    quote!( .and(#s.eq( key . #is)))  }).collect(); */
            
            let key_setters = &self.key_setters;
            /* Vec<proc_macro2::TokenStream> =
                self.key_setters.iter().enumerate().map(|x| { 
                                    let s = x.1;
                                    let i = x.0;  
                                    let is = syn::Index::from(i);
                                    quote!( #s = key . #is;)  }).collect(); */
             

                let key_predicate_fn = Ident::new(&format!("{}_key_predicate", &struct_ident).to_snake_case(), Span::call_site());
                let vis= self.vis;

                let struct_key_ident = Ident::new(&format!("{}Key", &struct_ident ), Span::call_site());
                let key_columns_code= &self.key_columns_code;

            quote! {

            #[derive(Debug, Eq, PartialEq, Hash)]
               #vis struct #struct_key_ident ( #key_type_code);

                impl toql::key::Key for #struct_ident {
                    type Key = #struct_key_ident;

                    fn get_key(&self) -> toql::error::Result<Self::Key> {
                       Ok(  #struct_key_ident (#key_getter) )
                    }
                    fn set_key(&mut self, key: Self::Key) -> toql::error::Result<()> {
                      #( #key_setters)*
                      Ok(())
                    }
                    fn columns() ->Vec<String> {
                         let mut columns: Vec<String>= Vec::new();

                        #(#key_columns_code)*
                        columns
                    }
                }
                
                #vis fn #key_predicate_fn (key: #struct_key_ident) ->Result<toql::query::Query , toql::error::ToqlError>{
                    Ok(toql::query::Query::new() #(#key_predicates)* ) 
                }

                /* impl toql::query_builder::Query< #key_type_code> for #struct_ident {
    
                    fn key_predicate<K>(key: K::Key) -> Result<toql::query::Query , toql::error::ToqlError>
                    where K : toql::key::Key< Key = #key_type_code>
                        {
                      
                      
                        Ok(toql::query::Query::new() #(#key_predicates)* ) 
                    }
                }
 */
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


                     fn select_one<C>(key: &<#struct_ident as toql::key::Key>::Key, conn: &mut C) 
                     -> Result<#struct_ident,  toql::error::ToqlError>
                     where C: toql::mysql::mysql::prelude::GenericConnection
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

                       
                        fn select_many<C>(
                            key: &<#struct_ident as toql::key::Key>::Key,
                            conn: &mut C,
                            first: u64,
                            max: u16
                        ) -> Result<Vec< #struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                        {
                                unimplemented!();


                        }

                        fn select_dependencies<C>(join: &str, params:&Vec<String>,
                            conn: &mut C) -> Result<Vec<#struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                            {
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