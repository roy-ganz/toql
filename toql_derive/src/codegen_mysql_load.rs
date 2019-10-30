/*
* Generation functions for toql derive
*
*/

use crate::annot::Toql;
use crate::annot::ToqlField;
use heck::MixedCase;
use heck::SnakeCase;
use proc_macro2::Span;

use syn::Ident;

use crate::sane::Struct;
use crate::sane::{Field, RegularField, JoinField, MergeField, FieldKind};

pub(crate) struct GeneratedMysqlLoad<'a> {
    struct_ident: &'a Ident,

    mysql_deserialize_fields: Vec<proc_macro2::TokenStream>,
    path_loaders: Vec<proc_macro2::TokenStream>,
    ignored_paths: Vec<proc_macro2::TokenStream>,
    merge_one_predicates: Vec<proc_macro2::TokenStream>,
    merge_many_predicates: Vec<proc_macro2::TokenStream>,
    forward_joins: Vec<proc_macro2::TokenStream>,
    regular_fields: usize, // Impl for mysql::row::ColumnIndex,
    merge_fields: Vec<crate::sane::Field>
}

impl<'a> GeneratedMysqlLoad<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlLoad {
        GeneratedMysqlLoad {
            struct_ident: &toql.rust_struct_ident,
            mysql_deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            merge_one_predicates: Vec::new(),
            merge_many_predicates: Vec::new(),
            forward_joins: Vec::new(),
            regular_fields: 0,
            merge_fields: Vec::new()
        }
    }

    pub(crate) fn add_mysql_deserialize_skip_field(&mut self, field: & crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_type_ident = &field.rust_type_ident;

        self.mysql_deserialize_fields.push(quote!(
             #rust_field_ident : #rust_type_ident :: default()
        ));
    }

    pub(crate) fn add_mysql_deserialize(&mut self,field: & crate::sane::Field) {
       // let field_ident = &field.ident;

        // Regular fields
        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                let rust_field_ident = &field.rust_field_ident;
                self.regular_fields += 1;

            let assignment = if self.mysql_deserialize_fields.is_empty() {
                quote!(*i)
            } else {
                quote!({
                    *i += 1;
                    *i
                })
            };

            let increment = if self.mysql_deserialize_fields.is_empty() {
                quote!()
            } else {
                quote!(*i += 1;)
            };

            // Check selection for optional Toql fields: Option<Option<..> or Option<..>
            if field.number_of_options > 0 && field.preselect == false {
                self.mysql_deserialize_fields.push(quote!(
                    #rust_field_ident : {
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
                    #rust_field_ident : row.take_opt( #assignment).unwrap()?
                ));
            }

            },
            FieldKind::Join(join_attrs) => {
                let rust_field_ident = &field.rust_field_ident;
                 let rust_type_ident = &field.rust_type_ident;
            self.forward_joins
                .push(quote!( i = < #rust_type_ident > ::forward_row(i);));
            let assignment = if self.mysql_deserialize_fields.is_empty() {
                quote!(i)
            } else {
                quote!({
                    *i += 1;
                    i
                })
            };

            let increment = if self.mysql_deserialize_fields.is_empty() {
                quote!(s)
            } else {
                quote!(*i += 1;)
            };
          

            // If any Option is present discriminator field must be added to check
            // - for unselected entity (discriminator column is NULL Type)
            // - for null entity (discriminator column is false) - only left joins

            self.mysql_deserialize_fields.push(
                 match   field.number_of_options {
                     2 =>   //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
                     quote!(
                                #rust_field_ident : {
                                       #increment
                                       if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                                            *i = < #rust_type_ident > ::forward_row(*i); // Added, but unsure, needs testing
                                        None
                                       }
                                       else if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                        //*i += 1; // Step over discriminator field,
                                        *i = < #rust_type_ident > ::forward_row(*i); 
                                        
                                        Some(None)
                                    } else {
                                        *i += 1;
                                        Some(Some(< #rust_type_ident > :: from_row_with_index ( & mut row , i )?))
                                    }
                                }
                        ),
                     1 if field.preselect =>   //    #[toql(preselect)] Option<T>  -> Nullable Join -> Left Join
                            quote!(
                                #rust_field_ident : {
                                     #increment
                                     if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                       // *i = < #join_type > ::forward_row({*i += 1; *i});
                                             *i = < #rust_type_ident > ::forward_row(*i);
                                        None
                                    } else {
                                        Some(< #rust_type_ident > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                    }
                                }
                            ),
                         1 if !field.preselect =>  //    Option<T>                         -> Selectable Join -> Inner Join
                                    quote!(
                                    #rust_field_ident : {
                                        #increment
                                        if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                                            *i = < #rust_type_ident > ::forward_row(*i);
                                            None
                                        } else {
                                        Some(< #rust_type_ident > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                        }
                                    }
                                ),
                     _ =>   //    T                                 -> Selected Join -> InnerJoin
                     quote!(
                #rust_field_ident :  < #rust_type_ident > :: from_row_with_index ( & mut row , #assignment )?
            )
                 }
             );


            },
            FieldKind::Merge(merge_attrs) =>  {
                  let rust_field_ident = &field.rust_field_ident;
                self.mysql_deserialize_fields.push( if  field.number_of_options > 0 { 
                    quote!( #rust_field_ident : None)
                } else {
                    quote!( #rust_field_ident : Vec::new())
                }
            );

            }



        }
/* 
        if field.join.is_none() && field.merge.is_none() {
            self.regular_fields += 1;

            let assignment = if self.mysql_deserialize_fields.is_empty() {
                quote!(*i)
            } else {
                quote!({
                    *i += 1;
                    *i
                })
            };

            let increment = if self.mysql_deserialize_fields.is_empty() {
                quote!()
            } else {
                quote!(*i += 1;)
            };

            // Check selection for optional Toql fields: Option<Option<..> or Option<..>
            if field.number_of_options() > 0 && field.preselect == false {
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
        else if field.join.is_some() {
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

            let increment = if self.mysql_deserialize_fields.is_empty() {
                quote!(s)
            } else {
                quote!(*i += 1;)
            };

            // There are 4 situations with joined entities
            //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
            //    #[toql(preselect)] Option<T>  -> Nullable Join -> Left Join
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
                                            *i = < #join_type > ::forward_row(*i); // Added, but unsure, needs testing
                                        None
                                       }
                                       else if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                        //*i += 1; // Step over discriminator field,
                                        *i = < #join_type > ::forward_row(*i); 
                                        
                                        Some(None)
                                    } else {
                                        *i += 1;
                                        Some(Some(< #join_type > :: from_row_with_index ( & mut row , i )?))
                                    }
                                }
                        ),
                     1 if field.preselect =>
                            quote!(
                                #field_ident : {
                                     #increment
                                     if row.take_opt::<bool,_>(*i).unwrap()? == false {
                                       // *i = < #join_type > ::forward_row({*i += 1; *i});
                                             *i = < #join_type > ::forward_row(*i);
                                        None
                                    } else {
                                        Some(< #join_type > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                    }
                                }
                            ),
                         1 if !field.preselect =>
                                    quote!(
                                    #field_ident : {
                                        #increment
                                        if row.columns_ref()[*i].column_type() == mysql::consts::ColumnType::MYSQL_TYPE_NULL {
                                            *i = < #join_type > ::forward_row(*i);
                                            None
                                        } else {
                                        Some(< #join_type > :: from_row_with_index ( & mut row , {*i += 1; i} )?)
                                        }
                                    }
                                ),
                     _ => quote!(
                #field_ident :  < #join_type > :: from_row_with_index ( & mut row , #assignment )?
            )
                 }
             );
        }
        // Merged fields
        else {
            self.mysql_deserialize_fields.push( if  field.number_of_options() > 0 { 
                    quote!( #field_ident : None)
                } else {
                    quote!( #field_ident : Vec::new())
                }
            );
        } */
    }
    pub(crate) fn add_merge_predicates(&mut self, toql: &Toql, field: &'a ToqlField) {
        let field_name = &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();
        //let vk :Vec<&str>= field.merge.as_ref().expect("Merge self struct field <= other struct field").split("<=").collect();
        //let toql_merge_field =format!("{}_{}",toql_field, vk.get(1).unwrap().trim().to_mixed_case());
        //let merge_struct_key_ident = Ident::new( vk.get(0).unwrap().trim(), Span::call_site());
        let field_type = field.first_non_generic_type().unwrap();
         let sql_merge_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
         let sql_merge_table_ident = Ident::new(&sql_merge_table_name, Span::call_site());

        for merge in &field.merge {
           // let toql_merge_field = format!("{}_{}", toql_field, merge.other_field.to_mixed_case());
            let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
            let auto_self_field= "id".to_string();

            let merge_struct_key_ident = Ident::new(&merge.this_field.as_ref().unwrap_or(&auto_self_field), Span::call_site());
            
            let other_column = crate::util::rename(&merge.other_field.as_ref().unwrap_or(&auto_other_field).to_string(), &toql.columns);
            

            let merge_one = format!("{{}}.{} = ?", other_column);
            let merge_many = format!("{{}}.{} IN ({{}})", other_column);

            let additional_merge_predicate = if merge.on_sql.is_some() {
                let merge_on= merge.on_sql.as_ref().unwrap();

                let (merge_with_params, merge_params) = crate::util::extract_query_params(merge_on);
                // if on_sql contains .. replace them with table alias
                let merge_on = if merge_with_params.contains("..") {
                        let aliased_merge_on = merge_with_params.replace("..", "{alias}.");
                        quote!(
                            format!(#aliased_merge_on, alias = <#sql_merge_table_ident as toql::sql_mapper::Mapped>::table_alias() )
                        )
                } else {
                    quote!( #merge_with_params)
                };
                
                let params = merge_params.iter().map(|p| {
                    quote!( query.where_predicate_params.push( query.params
                                .get(  #p)
                                .ok_or(toql::sql_builder::SqlBuilderError::QueryParamMissing(#p))?);
                        )   
                }).collect::<proc_macro2::TokenStream>();

                quote!( 
                    query.where_predicates.push(#merge_on);

                    #(#params)*
                    
                )
            } else {
                quote!()
            };
             

            /* self.merge_one_predicates.push( quote!(
                query.where_predicates.push( format!(#merge_one, <#sql_merge_table_ident as toql::sql_mapper::Mapped>::table_alias()));
                query.where_predicate_params.push(_entity. #merge_struct_key_ident .to_string());
                #additional_merge_predicate
            )); */
            self.merge_many_predicates.push( quote!(
                let q = entities.iter().map(|entity| "?" ).collect::<Vec<&str>>().join(", ");
                dep_query.where_predicates.push(format!(#merge_many, <#sql_merge_table_ident as toql::sql_mapper::Mapped>::table_alias(), q));
                dep_query.where_predicate_params.extend_from_slice(entities.iter().map(|entity| entity. #merge_struct_key_ident .to_string()).collect::<Vec<String>>().as_ref());
                #additional_merge_predicate
            ));

            /* self.merge_one_predicates.push( quote!(
                   let query = query.and(toql::query::Field::from(#toql_merge_field).eq( _entity. #merge_struct_key_ident));
            ));

            self.merge_many_predicates.push( quote!(
                  let query =  query.and(toql::query::Field::from(#toql_merge_field).ins(entities.iter().map(|entity| entity. #merge_struct_key_ident).collect()));
            )); 
        }
        */
        */
    }
    pub(crate) fn add_merge_predicates(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.clone());

    }
    pub(crate) fn add_ignored_path(&mut self, field: &crate::sane::Field) {
        
        let toql_field = &field.toql_field_name;

        self.ignored_paths.push(quote!(
                    .ignore_path( #toql_field)));
    }
    pub(crate) fn add_path_loader(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.clone());
    }
    /* fn build_path_loader(&mut self, toql: &Toql, field: &crate::sane::Field) {
        let struct_ident = &self.struct_ident;
        let field_ident = &field.rust_field_ident;
        let field_name = &field.rust_field_name;
        let toql_field = field_name.to_mixed_case();
        let merge_type = field.first_non_generic_type().unwrap();

        // handle merge keys fields and on_sql 
         let sql_merge_table_name = crate::util::rename(&merge_type.to_string(), &toql.tables);
         let sql_merge_table_ident = Ident::new(&sql_merge_table_name, Span::call_site());

        let mut merge_many_predicates :Vec<proc_macro2::TokenStream> = Vec::new();

        for merge in &field.merge {
           // let toql_merge_field = format!("{}_{}", toql_field, merge.other_field.to_mixed_case());
            let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
            let auto_self_field= "id".to_string();

            let merge_struct_key_ident = Ident::new(&merge.this_field.as_ref().unwrap_or(&auto_self_field), Span::call_site());
            
            let other_column = crate::util::rename(&merge.other_field.as_ref().unwrap_or(&auto_other_field).to_string(), &toql.columns);
            

            let merge_one = format!("{{}}.{} = ?", other_column);
            let merge_many = format!("{{}}.{} IN ({{}})", other_column);

            let additional_merge_predicate = if merge.on_sql.is_some() {
                let merge_on= merge.on_sql.as_ref().unwrap();

                let (merge_with_params, merge_params) = crate::util::extract_query_params(merge_on);
                // if on_sql contains .. replace them with table alias
                let merge_on = if merge_with_params.contains("..") {
                        let aliased_merge_on = merge_with_params.replace("..", "{alias}.");
                        quote!(
                            format!(#aliased_merge_on, alias = <#sql_merge_table_ident as toql::sql_mapper::Mapped>::table_alias() )
                        )
                } else {
                    quote!( #merge_with_params)
                };
                
                let params = merge_params.iter().map(|p| {
                    quote!( dep_query.where_predicate_params.push( query.params
                                .get(  #p)
                                .ok_or(toql::sql_builder::SqlBuilderError::QueryParamMissing(#p .to_string()))? .to_owned() );
                        )   
                }).collect::<proc_macro2::TokenStream>();

                quote!( 
                    dep_query.where_predicates.push(#merge_on);

                    #(#params)*
                    
                )
            } else {
                quote!()
            };
             merge_many_predicates.push( quote!(
                let q = entities.iter().map(|entity| "?" ).collect::<Vec<&str>>().join(", ");
                dep_query.where_predicates.push(format!(#merge_many, <#sql_merge_table_ident as toql::sql_mapper::Mapped>::table_alias(), q));
                dep_query.where_predicate_params.extend_from_slice(entities.iter().map(|entity| entity. #merge_struct_key_ident .to_string()).collect::<Vec<String>>().as_ref());
                #additional_merge_predicate
            ));
        }
             


        

        let merge_function = Ident::new(
            &format!("merge_{}", &field.ident.as_ref().unwrap()),
            Span::call_site(),
        );

       let merge_field_init = if field.number_of_options() > 0 {
                 quote!( Some(Vec::new())) 
            } else {
                quote!(Vec::new()) 
            };

        let sql_merge_table_ident = Ident::new(&sql_merge_table_name, Span::call_site());

        self.path_loaders.push( quote!(

                let mut dep_query = query.clone();
                #(#merge_many_predicates)*
               
                let #field_ident = #merge_type ::load_path_from_mysql(#toql_field, &dep_query, cache, conn)?;
                if #field_ident .is_some() {
                    for e in entities.iter_mut() { e . #field_ident = #merge_field_init; }
                    #struct_ident :: #merge_function (&mut entities, #field_ident .unwrap());
                }
         ));
    } */
    pub(crate) fn loader_functions(&self) -> proc_macro2::TokenStream {
        let struct_ident = &self.struct_ident;
        let struct_name = &self.struct_ident.to_string();
        let path_loaders = &self.path_loaders;
        let ignored_paths = &self.ignored_paths;
        let merge_one_predicates = &self.merge_one_predicates;
        let merge_many_predicates = &self.merge_many_predicates;

        let load_dependencies_from_mysql = if path_loaders.is_empty() {
            quote!(
                pub fn load_dependencies_from_mysql<C>(mut _entities: &mut Vec< #struct_ident >,
                _query: &toql::query::Query,  _cache: &toql::sql_mapper::SqlMapperCache, _conn: &mut C)
                -> toql::error::Result<()> 
                where C: toql::mysql::mysql::prelude::GenericConnection
                { Ok(())}
            )
        } else {
            quote!(
                pub fn load_dependencies_from_mysql<C>(mut entities: &mut Vec< #struct_ident >,
                query: &toql::query::Query,  cache: &toql::sql_mapper::SqlMapperCache, conn: &mut C)
                -> toql::error::Result<()>
                where C: toql::mysql::mysql::prelude::GenericConnection
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
                //let mut query = query.clone();
                //let _entity = entities.get(0).unwrap();
                //#(#merge_one_predicates)*
                #struct_ident ::load_dependencies_from_mysql(&mut entities, &query, cache, conn)?;
            )
        };
        let load_many_call_dependencies = if path_loaders.is_empty() {
            quote!()
        } else {
            quote!(
                let mut query = query.clone();
                // Resolve dependencies
                // Restrict query to keys
                //#(#merge_many_predicates)*

                #struct_ident ::load_dependencies_from_mysql(&mut entities, &query, cache, conn)?;
            )
        };

        quote!(
            impl #struct_ident {

                pub fn load_path_from_mysql<C>(path: &str, query: &toql::query::Query, 
                    cache: &toql::sql_mapper::SqlMapperCache,  
                    conn: &mut C)
                -> toql::error::Result<Option<std::vec::Vec< #struct_ident >>>
                where C: toql::mysql::mysql::prelude::GenericConnection
                {
                    let mapper = cache.mappers.get( #struct_name ).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    let result = toql::sql_builder::SqlBuilder::new().build_path(path, mapper, &query)?;
                    toql::log_sql!( result.to_sql(),result.params());
                    
                    if result.is_empty() {
                        Ok(None)
                    } else {
                        let entities_stmt = conn.prep_exec(result.to_sql(), result.params())?;
                        let entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
                        Ok(Some(entities))
                    }
                }


                #load_dependencies_from_mysql
            }
            impl toql::mysql::load::Load<#struct_ident> for #struct_ident
            {
                fn load_one<C>(query: &toql::query::Query, 
                    cache: &toql::sql_mapper::SqlMapperCache, 
                    conn: &mut C )
                    -> toql::error::Result<# struct_ident>
                    where C: toql::mysql::mysql::prelude::GenericConnection
                {
                    let mapper = cache.mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;



                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query)?;

                    toql::log_sql!(result.to_sql_for_mysql("", 0, 2), result.params());
                    



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


                fn load_many<C>(query: &toql::query::Query, 
                cache: &toql::sql_mapper::SqlMapperCache,
                conn: &mut C, count:bool, first:u64, max:u16)
                -> toql::error::Result<(std::vec::Vec< #struct_ident >, Option<(u32, u32)>)> 
                where C: toql::mysql::mysql::prelude::GenericConnection
                {

                    let mapper = cache.mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    // load base entities

                    let hint = String::from( if count {"SQL_CALC_FOUND_ROWS" }else{""});

                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query)?;

                    toql::log_sql!(result.to_sql_for_mysql(&hint, first, max), result.params());
                    
                    let entities_stmt = conn.prep_exec(result.to_sql_for_mysql( &hint, first, max), result.params())?;
                    let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
                    let mut count_result = None;

                    // Get count values
                    if count {
                          toql::log_sql!("SELECT FOUND_ROWS();");
                        //toql::log::info!("SQL `SELECT FOUND_ROWS();`");
                        let r = conn.query("SELECT FOUND_ROWS();")?;
                        let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();

                        let result = toql::sql_builder::SqlBuilder::new().build_count(mapper, &query)?;
                        toql::log_sql!(result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params());
                        conn.prep_exec(result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params())?; // Don't select any rows

                        toql::log_sql!("SELECT FOUND_ROWS();");
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

impl<'a> quote::ToTokens for GeneratedMysqlLoad<'a> {
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

        log::debug!(
            "Source code for `{}`:\n{}",
            &self.struct_ident,
            mysql.to_string()
        );

        tokens.extend(mysql);
    }
}
