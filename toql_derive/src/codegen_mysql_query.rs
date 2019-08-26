/*
* Generation functions for toql derive
*
*/

use crate::annot::Toql;
use crate::annot::ToqlField;
use heck::MixedCase;
use proc_macro2::Span;

use syn::Ident;


pub(crate) struct GeneratedMysqlQuery<'a> {
    struct_ident: &'a Ident,
    

    mysql_deserialize_fields: Vec<proc_macro2::TokenStream>,
    path_loaders: Vec<proc_macro2::TokenStream>,
    ignored_paths: Vec<proc_macro2::TokenStream>,
    merge_one_predicates: Vec<proc_macro2::TokenStream>,
    merge_many_predicates: Vec<proc_macro2::TokenStream>,
    forward_joins: Vec<proc_macro2::TokenStream>,
    regular_fields: usize, // Impl for mysql::row::ColumnIndex
}

impl<'a> GeneratedMysqlQuery<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedMysqlQuery {

        GeneratedMysqlQuery {
            struct_ident: &toql.ident,
            mysql_deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            merge_one_predicates: Vec::new(),
            merge_many_predicates: Vec::new(),
            forward_joins: Vec::new(),
            regular_fields: 0,
        }
    }

    
    pub(crate) fn add_mysql_deserialize_skip_field(&mut self, field: &'a ToqlField) {
        let field_ident = &field.ident;
        let field_type = &field.ty;
        self.mysql_deserialize_fields.push(quote!(
             #field_ident : #field_type :: default()
        ));
    }

    pub(crate) fn add_mysql_deserialize(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let field_ident = &field.ident;

        // Regular fields
        if field.sql_join.is_empty() && field.merge.is_empty() {
            self.regular_fields += 1;

            let assignment = if self.mysql_deserialize_fields.is_empty() {
                quote!(*i)
            } else {
                quote!({
                    *i += 1;
                    *i
                })
            };

            let increment =  if self.mysql_deserialize_fields.is_empty() { quote!()
            } else {
                quote!(*i += 1;)
            };

            // Check selection for optional Toql fields: Option<Option<..> or Option<..> 
            if field.number_of_options() > 0 && field.select_always == false {
                self.mysql_deserialize_fields.push(quote!(
                    #field_ident : {
                        #increment
                        if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                            None
                        } else {
                            row.take_opt( *i).unwrap()?
                        }
                    }
                    
                ));
            } else {
                self.mysql_deserialize_fields.push(quote!(
                    #field_ident : row.take_opt( #assignment).unwrap()?
                ));
            }
        }
        // Joined fields
        else if !field.sql_join.is_empty() {
            let join_type = field.first_non_generic_type();
            self.forward_joins
                .push(quote!( i = < #join_type > ::forward_row(i);));
            let assignment = if self.mysql_deserialize_fields.is_empty() {
                quote!(i)
            } else {
                quote!({
                    *i += 1;
                    i
                })
            };

            let increment =  if self.mysql_deserialize_fields.is_empty() { quote!(s)
            } else {
                quote!(*i += 1;)
            };

            // There are 4 situations with joined entities
            //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
            //    #[toql(select_always)] Option<T>  -> Nullable Join -> Left Join
            //    Option<T>                         -> Selectable Join -> Inner Join
            //    T                                 -> Selected Join -> InnerJoin       

            // If any Option is present discriminator field must be added to check
            // - for unselected entity (discriminator column is NULL Type)
            // - for null entity (discriminator column is false) - only left joins
          
             self.mysql_deserialize_fields.push( 
                 match   field.number_of_options() {
                     2 =>  quote!(
                                #field_ident : {  
                                       #increment
                                       if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                                        None
                                       } 
                                       else if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                        *i += 1;
                                        *i = < #join_type > ::forward_row(*i);
                                        Some(None)
                                    } else {
                                        *i += 1;
                                        Some(Some(< #join_type > :: from_row_with_index ( & mut row , i )?))
                                    }
                                }
                        ),
                     1 => {
                         if field.select_always {
                            quote!(
                                #field_ident : { 
                                     #increment
                                     
                                     if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                        *i = < #join_type > ::forward_row({*i += 1; *i});
                                        None
                                    } else {
                                        Some(< #join_type > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                    }
                                }
                            )
                         } else {
                                quote!(
                                #field_ident : { 
                                     #increment
                                     if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                                        None
                                    } else {
                                    Some(< #join_type > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                    }
                                }
                        )
                         }
                         

                     },
                     _ => quote!( 
                #field_ident :  < #join_type > :: from_row_with_index ( & mut row , #assignment )?
            )
                 }
             );


/* 
                 }
                if field.number_of_options() > 0   {
                    // Join can be NONE
                    if field.number_of_options() == 2 
                    || field.number_of_options() == 1 && field.select_always == true {
                        quote!(
                                                        #field_ident : {  
                                                                if 
                                                                if row.take(#assignment) == false {
                                                                *i += 1;
                                                                i = < #join_type > ::forward_row(i);
                                                                Some(None)
                                                            } else {
                                                             Some(< #join_type > :: from_row_with_index ( & mut row , #assignment )?)
                                                            }
                                                        }
                                                )
                    } else {

                        quote!(
                                #field_ident : {  if row.take(#assignment) == false {
                                        *i += 1;
                                        i = < #join_type > ::forward_row(i);
                                        None
                                    } else {
                                    < #join_type > :: from_row_with_index ( & mut row , #assignment )?
                                    }
                                }
                        )
                    }
            } else { 
            quote!( 
                #field_ident :  < #join_type > :: from_row_with_index ( & mut row , #assignment )?
            )
            }
             );
/* */
            self.mysql_deserialize_fields.push( quote!(
                    #field_ident : {  if #none_assignment
                                    } else {
                                     
                                        let j = *i;
                                        let #field_ident = < #join_type > :: from_row_with_index ( & mut row , #assignment ).ok();
                                        *i = if #field_ident .is_none() { < #join_type > :: forward_row (j)} else {*i}; // Recover index from error
                                        #field_ident
                                    }
                            
                ));
                */
/*
            // If join is optional, assign None if deserialization fails
            if field._first_type() == "Option" {

                self.mysql_deserialize_fields.push( quote!(
                    #field_ident : {  if row.take(*i) == true { // Key is null, forward and return None
                                        i = < #join_type > ::forward_row(i);
                                        None
                                    } else {
                                     
                                    let j = *i;
                                    let #field_ident = < #join_type > :: from_row_with_index ( & mut row , #assignment ).ok();
                                    *i = if #field_ident .is_none() { < #join_type > :: forward_row (j)} else {*i}; // Recover index from error
                                    #field_ident
                                    }
                            }
                ));
            } else {
                self.mysql_deserialize_fields.push( quote!(
                    #field_ident :  < #join_type > :: from_row_with_index ( & mut row , #assignment ) ? 
                ));
            }
            */
        }
        // Merged fields
        else {
            self.mysql_deserialize_fields.push(quote!(
                #field_ident : Vec::new()
            ));
        }
    }
    pub(crate) fn add_merge_predicates(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let field_name = &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();
        //let vk :Vec<&str>= field.merge.as_ref().expect("Merge self struct field <= other struct field").split("<=").collect();
        //let toql_merge_field =format!("{}_{}",toql_field, vk.get(1).unwrap().trim().to_mixed_case());
        //let merge_struct_key_ident = Ident::new( vk.get(0).unwrap().trim(), Span::call_site());

        for merge in &field.merge {
            let toql_merge_field = format!("{}_{}", toql_field, merge.other.to_mixed_case());
            let merge_struct_key_ident = Ident::new(&merge.this, Span::call_site());
            self.merge_one_predicates.push( quote!(
                        query.and(toql::query::Field::from(#toql_merge_field).eq( _entity. #merge_struct_key_ident));
            ));

            self.merge_many_predicates.push( quote!(
                   query.and(toql::query::Field::from(#toql_merge_field).ins(entities.iter().map(|entity| entity. #merge_struct_key_ident).collect()));
            ));
        }
    }
    pub(crate) fn add_ignored_path(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let field_name = &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();

        self.ignored_paths.push(quote!(
                    .ignore_path( #toql_field)));
    }
    pub(crate) fn add_path_loader(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let struct_ident = &self.struct_ident;
        let field_ident = &field.ident;
        let field_name = &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();
        let merge_type = field.first_non_generic_type().unwrap();

        let merge_function = Ident::new(
            &format!("merge_{}", &field.ident.as_ref().unwrap()),
            Span::call_site(),
        );

        self.path_loaders.push( quote!(
                let #field_ident = #merge_type ::load_path_from_mysql(#toql_field, &query, mappers, conn)?;
                #struct_ident :: #merge_function (&mut entities, #field_ident);
         ));
    }
    pub(crate) fn loader_functions(&self) -> proc_macro2::TokenStream {
        let struct_ident = &self.struct_ident;
        let struct_name = &self.struct_ident.to_string();
        let path_loaders = &self.path_loaders;
        let ignored_paths = &self.ignored_paths;
        let merge_one_predicates = &self.merge_one_predicates;
        let merge_many_predicates = &self.merge_many_predicates;

        let load_dependencies_from_mysql = if path_loaders.is_empty() {
            quote!(
                pub fn load_dependencies_from_mysql(mut _entities: &mut Vec< #struct_ident >,
                _query: &toql::query::Query,  _mappers: &toql::sql_mapper::SqlMapperCache, _conn: &mut toql::mysql::mysql::Conn)
                -> toql::error::Result<()> { Ok(())}
            )
        } else {
            quote!(
                pub fn load_dependencies_from_mysql(mut entities: &mut Vec< #struct_ident >,
                query: &toql::query::Query,  mappers: &toql::sql_mapper::SqlMapperCache, conn: &mut toql::myql::mysql::Conn)
                -> toql::error::Result<()>
                {
                    #(#path_loaders)*
                    Ok(())
                }
            )
        };

        let load_one_call_dependencies = if path_loaders.is_empty() {
            quote!()
        } else {
            quote!(
             // Restrict dependencies to parent entity
                // query.and( "parent_child_id eq XX" )
                let mut query = query.clone();
                let _entity = entities.get(0).unwrap();
                #(#merge_one_predicates)*
                #struct_ident ::load_dependencies_from_mysql(&mut entities, &query, mappers, conn)?;
            )
        };
        let load_many_call_dependencies = if path_loaders.is_empty() {
            quote!()
        } else {
            quote!(
                let mut query = query.clone();
                // Resolve dependencies
                // Restrict query to keys
                #(#merge_many_predicates)*

                #struct_ident ::load_dependencies_from_mysql(&mut entities, &query, mappers, &mut conn)?;
            )
        };
        
        

        quote!(
            impl #struct_ident {

                pub fn load_path_from_mysql(path: &str, query: &toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache,  conn: &mut toql::mysql::mysql::Conn)
                -> toql::error::Result<std::vec::Vec< #struct_ident >>
                {
                    let mapper = mappers.get( #struct_name ).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    let result = toql::sql_builder::SqlBuilder::new().build_path(path, mapper, &query)?;
                    toql::log::info!("SQL `{}` with params {:?}", result.to_sql(), result.params());
                    if result.is_empty() {
                        Ok(vec![])
                    } else {
                        let entities_stmt = conn.prep_exec(result.to_sql(), result.params())?;
                        let entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
                        Ok(entities)
                    }
                }


                #load_dependencies_from_mysql
            }
            impl toql::mysql::load::Load<#struct_ident> for #struct_ident
            {
                fn load_one(query: &toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache, conn: &mut toql::mysql::mysql::Conn )
                    -> toql::error::Result<# struct_ident>
                {
                    let mapper= mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;

                  

                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query)?;

                    toql::log::info!("SQL `{}` with params {:?}", result.to_sql_for_mysql("", 0, 2), result.params());



                    let entities_stmt = conn.prep_exec(result.to_sql_for_mysql( "", 0, 2), result.params())?;
                    let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;

                    if entities.len() > 1 {
                        return Err(toql::error::ToqlError::NotUnique);
                    } else if entities.is_empty() {
                        return Err(toql::error::ToqlError::NotFound);
                    }

                    #load_one_call_dependencies
                    Ok(entities.pop().unwrap())
                }


                fn load_many(query: &toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache,
                mut conn: &mut toql::mysql::mysql::Conn, count:bool, first:u64, max:u16)
                -> toql::error::Result<(std::vec::Vec< #struct_ident >, Option<(u32, u32)>)> {

                    let mapper = mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    // load base entities

                    let hint = String::from( if count {"SQL_CALC_FOUND_ROWS" }else{""});

                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query)?;

                    toql::log::info!("SQL `{}` with params {:?}", result.to_sql_for_mysql(&hint, first, max), result.params());
                    let entities_stmt = conn.prep_exec(result.to_sql_for_mysql( &hint, first, max), result.params())?;
                    let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
                    let mut count_result = None;

                    // Get count values
                    if count {
                        toql::log::info!("SQL `SELECT FOUND_ROWS();`");
                        let r = conn.query("SELECT FOUND_ROWS();")?;
                        let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();

                        let result = toql::sql_builder::SqlBuilder::new().build_count(mapper, &query)?;
                        toql::log::info!("SQL `{}` with params {:?}", result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params());
                        conn.prep_exec(result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params())?; // Don't select any rows
                        toql::log::info!("SQL `SELECT FOUND_ROWS();`");
                        let r = conn.query("SELECT FOUND_ROWS();")?;
                        let filtered_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
                        count_result = Some((total_count ,filtered_count))
                    }

                   #load_many_call_dependencies

                    Ok((entities, count_result))
                }
            }

        )
    }
}

impl<'a> quote::ToTokens for GeneratedMysqlQuery<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;
        let loader = self.loader_functions();

        let mysql_deserialize_fields = &self.mysql_deserialize_fields;

        let regular_fields = self.regular_fields;
        let forward_joins = &self.forward_joins;

              

        let mysql = quote!(

            #loader


            impl toql :: mysql :: row:: FromResultRow < #struct_ident > for #struct_ident {
            fn forward_row(mut i : usize) -> usize {
                i += #regular_fields ;
                #(#forward_joins)*
                i
            }

            fn from_row_with_index ( mut row : & mut toql::mysql::mysql :: Row , i : &mut usize) -> std::result::Result < #struct_ident , toql::mysql::mysql :: error :: Error > {

                Ok ( #struct_ident {
                    #(#mysql_deserialize_fields),*

                })
            }
            }


        );
        
        
        log::debug!("Source code for `{}`:\n{}", &self.struct_ident, mysql.to_string());

        tokens.extend(mysql);
    }
}
