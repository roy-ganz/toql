/*
* Generation functions for toql derive
*
*/

use proc_macro2::Span;

use proc_macro2::TokenStream;
use syn::Ident;

use crate::sane::FieldKind;
use crate::sane::MergeColumn;
use crate::sane::Struct;
use std::collections::HashMap;

pub(crate) struct GeneratedMysqlLoad<'a> {
    rust_struct: &'a Struct,

    mysql_deserialize_fields: Vec<TokenStream>,
    path_loaders: Vec<TokenStream>,
    ignored_paths: Vec<TokenStream>,

    forward_joins: Vec<TokenStream>,
    regular_fields: usize, // Impl for mysql::row::ColumnIndex,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
    merge_field_getter: HashMap<String, TokenStream>,
}

impl<'a> GeneratedMysqlLoad<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlLoad {
        GeneratedMysqlLoad {
            rust_struct: &toql,
            mysql_deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            forward_joins: Vec::new(),
            regular_fields: 0,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
            merge_field_getter: HashMap::new(),
        }
    }

    pub(crate) fn add_mysql_deserialize_skip_field(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_type_ident = &field.rust_type_ident;

        self.mysql_deserialize_fields.push(quote!(
             #rust_field_ident : #rust_type_ident :: default()
        ));
    }

    pub(crate) fn add_mysql_deserialize(&mut self, field: &crate::sane::Field) {
        // Regular fields
        let rust_field_name = &field.rust_field_name;
        let error_field = format!(
            "{}::{}",
            &self.rust_struct.rust_struct_ident, rust_field_name
        );
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
                            row.take_opt( *i).unwrap()
                                .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                        }
                    }
                ));
                } else {
                    self.mysql_deserialize_fields.push(quote!(
                        #rust_field_ident : row.take_opt( #assignment).unwrap()
                            .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                    ));
                }

                if regular_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());

                    // Merge field getter for non null columns
                    match field.number_of_options {
                        0 => {
                            self.merge_field_getter.insert(
                                rust_field_name.to_string(),
                                quote!(entity. #rust_field_ident. to_string()),
                            );
                        }
                        1 if !field.preselect => {
                            self.merge_field_getter.insert(rust_field_name.to_string(),
                            quote!(entity. #rust_field_ident .as_ref()
                                .ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?.to_string()));
                        }
                        _ => {}
                    };
                }
            }
            FieldKind::Join(join_attrs) => {
                let rust_field_ident = &field.rust_field_ident;
                let rust_field_name = &field.rust_field_name;
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
                    quote!()
                } else {
                    quote!(*i += 1;)
                };

                // For optional joined fields (left Joins) a discriminator field must be added to check
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
                                       else if row.take_opt::<bool,_>(*i).unwrap()
                                        .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                                        == false {
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
                                     if row.take_opt::<bool,_>(*i).unwrap()
                                      .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                                      == false {
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
                                            *i = < #rust_type_ident > ::forward_row(*i) - 1; // No discriminator field
                                            None
                                        } else {
                                        Some(< #rust_type_ident > :: from_row_with_index ( & mut row , i )?)
                                        }
                                    }
                                ),
                     _ =>   //    T                                 -> Selected Join -> InnerJoin
                     quote!(
                        #rust_field_ident :  < #rust_type_ident > :: from_row_with_index ( & mut row , #assignment )?
                    )
                 }
            );

                if join_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Merge(_merge_attrs) => {
                let rust_field_ident = &field.rust_field_ident;
                self.mysql_deserialize_fields
                    .push(if field.number_of_options > 0 {
                        quote!( #rust_field_ident : None)
                    } else {
                        quote!( #rust_field_ident : Vec::new())
                    });
            }
        }
    }

    pub(crate) fn add_ignored_path(&mut self, field: &crate::sane::Field) {
        let toql_field = &field.toql_field_name;

        self.ignored_paths.push(quote!(
                    .ignore_path( #toql_field)));
    }

    pub(crate) fn add_path_loader(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.clone());
    }

    pub(crate) fn loader_functions(&self) -> proc_macro2::TokenStream {
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;
        let path_loaders = &self.path_loaders;
        let ignored_paths = &self.ignored_paths;
        let struct_key_ident = Ident::new(&format!("{}Key", struct_ident), Span::call_site());
        let load_dependencies_from_mysql = if path_loaders.is_empty() {
            quote!(
               /*  fn load_dependencies(&mut self, mut _entities: &mut Vec< #struct_ident >,
                _query: &toql::query::Query,  _cache: &toql::sql_mapper::SqlMapperCache)
                -> Result<(), toql::mysql::error::ToqlMySqlError>
                { Ok(())} */
            )
        } else {
            quote!(
                fn load_dependencies(&mut self, mut entities: &mut Vec< #struct_ident>,mut entity_keys: & Vec< #struct_key_ident >,
                query: &toql::query::Query,  cache: &toql::sql_mapper::SqlMapperCache)
                -> Result<(), toql::mysql::error::ToqlMySqlError>
                {
                    //let mapper = cache.mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    //let conn = self.conn();
                    #(#path_loaders)*
                    Ok(())
                }
            )
        };

        let optional_add_primary_keys = if self.merge_fields.is_empty() {
            quote!()
        } else {
            quote!(
                 <#struct_ident as toql::key::Key>::columns().iter().for_each(|c|{
                    result.push_select(&mapper.aliased_column(&<#struct_ident as toql::sql_mapper::Mapped>::table_alias(),&c))
                });
            )
        };

        let from_query_result = if self.merge_fields.is_empty() {
            quote!(
                let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
            )
        } else {
            quote!(
            let (mut entities, keys) = toql::mysql::row::from_query_result_with_primary_keys::<#struct_ident, #struct_key_ident>(entities_stmt)?;
            )
        };

        let optional_load_merges = if self.merge_fields.is_empty() {
            quote!()
        } else {
            quote!(
            self.load_dependencies(&mut entities, &keys, &query, cache)?;
            )
        };

        quote!(

            impl<'a, T: toql::mysql::mysql::prelude::GenericConnection + 'a> toql::load::Load<#struct_ident> for toql::mysql::MySql<'a,T>
            {
                type error = toql :: mysql::error::ToqlMySqlError;

                fn load_one(&mut self, query: &toql::query::Query,

                    cache: &toql::sql_mapper::SqlMapperCache,
                   )
                    -> Result<# struct_ident, toql :: mysql::error:: ToqlMySqlError>

                {
                  //   let conn = self.conn();
                    let mapper = cache.mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;

                    let mut result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query, self.roles()).map_err(|e|toql::error::ToqlError::SqlBuilderError(e))?;

                    #optional_add_primary_keys

                    toql::log_sql!(toql::mysql::sql_from_query_result( &result, "", 0, 2), result.params());




                    let entities_stmt = self.conn().prep_exec(toql::mysql::sql_from_query_result( &result, "", 0, 2), result.params())?;
                    #from_query_result
                    //let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;

                    if entities.len() > 1 {
                        return Err(toql::mysql::error::ToqlMySqlError::ToqlError(toql::error::ToqlError::NotUnique));
                    } else if entities.is_empty() {
                        return Err(toql::mysql::error::ToqlMySqlError::ToqlError(toql::error::ToqlError::NotFound));
                    }

                    #optional_load_merges
                    Ok(entities.pop().unwrap())
                }


                fn load_many( &mut self, query: &toql::query::Query,

                cache: &toql::sql_mapper::SqlMapperCache,
                page: toql::load::Page)
                -> Result<(std::vec::Vec< #struct_ident >, Option<(u32, u32)>), toql :: mysql::error:: ToqlMySqlError>
                {
                 //   let conn = self.conn();
                    let mut count = false;
                    let mut first = 0;
                    let mut max = 10;
                    match page {
                        toql::load::Page::Counted(f, m) =>  {count = true; first = f; max = m},
                        toql::load::Page::Uncounted(f, m) =>  {count = false; first = f; max = m},
                    };

                    let mapper = cache.mappers.get( #struct_name)
                            .ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    // load base entities

                    let hint = String::from( if count {"SQL_CALC_FOUND_ROWS" }else{""});

                    let mut result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query, self.roles()).map_err(|e|toql::error::ToqlError::SqlBuilderError(e))?;

                    #optional_add_primary_keys


                    toql::log_sql!(toql::mysql::sql_from_query_result( &result, &hint, first, max), result.params());
                    let entities_stmt = self.conn().prep_exec(toql::mysql::sql_from_query_result( &result, &hint, first, max), result.params())?;
                    #from_query_result
                    //let mut entities = toql::mysql::row::from_query_result::< #struct_ident >(entities_stmt)?;
                    let mut count_result = None;

                    // Get count values
                    if count {
                        toql::log_sql!("SELECT FOUND_ROWS();");
                        let r = self.conn().query("SELECT FOUND_ROWS();")?;
                        let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();

                        let result = toql::sql_builder::SqlBuilder::new()
                        #(#ignored_paths)*
                        .build_count(mapper, &query, self.roles()).map_err(|e|toql::error::ToqlError::SqlBuilderError(e))?;
                        toql::log_sql!(toql::mysql::sql_from_query_result( &result, "SQL_CALC_FOUND_ROWS", 0, 0), result.params());
                        self.conn().prep_exec(toql::mysql::sql_from_query_result( &result,"SQL_CALC_FOUND_ROWS", 0, 0), result.params())?; // Don't select any rows

                        toql::log_sql!("SELECT FOUND_ROWS();");
                        let r = self.conn().query("SELECT FOUND_ROWS();")?;
                        let filtered_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
                        count_result = Some((total_count ,filtered_count))
                    }

                   #optional_load_merges

                    Ok((entities, count_result))
                }
                fn build_path(&mut self,path: &str, query: &toql::query::Query,
                    cache: &toql::sql_mapper::SqlMapperCache
                   )
                -> Result<toql::sql_builder_result::SqlBuilderResult,toql :: mysql::error:: ToqlMySqlError>

                {
                    let mapper = cache.mappers.get( #struct_name ).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                    Ok( toql::sql_builder::SqlBuilder::new()
                        .build_path(path, mapper, &query, self.roles())
                        .map_err(|e|toql::error::ToqlError::SqlBuilderError(e))?)
                }

                #load_dependencies_from_mysql
            }

        )
    }

    pub fn build_merge(&mut self) {
        // Build all merge fields
        // This must be done after the first pass, becuase all key names must be known at this point
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;
        for field in &self.merge_fields {
            let rust_type_ident = &field.rust_type_ident;
            let rust_field_ident = &field.rust_field_ident;
            let toql_field_name = &field.toql_field_name;

            match &field.kind {
                FieldKind::Merge(merge_attrs) => {
                    let mut inverse_column_translation: Vec<TokenStream> = Vec::new();

                    for m in &merge_attrs.columns {
                        let untranslated_column = &m.this;
                        match &m.other {
                            MergeColumn::Unaliased(other_column) => {
                                inverse_column_translation.push( quote!( #untranslated_column => mapper.aliased_column(&<#rust_type_ident as toql::sql_mapper::Mapped>::table_alias(),#other_column), ));
                            }
                            MergeColumn::Aliased(other_column) => {
                                inverse_column_translation.push(
                                    quote!( #untranslated_column => String::from(#other_column),),
                                );
                            }
                        };
                    }

                    let merge_field_init = if field.number_of_options > 0 {
                        quote!(e . #rust_field_ident = Some(Vec::new()))
                    } else {
                        quote!(e . #rust_field_ident = Vec::new())
                    };
                    let merge_field_assign = if field.number_of_options > 0 {
                        quote!(if e. #rust_field_ident. is_some() {
                            e. #rust_field_ident. as_mut().unwrap().push(m);
                        })
                    } else {
                        quote!(e.  #rust_field_ident. push(m);)
                    };

                    let path_test = if field.number_of_options > 0 && !field.preselect {
                        quote!( if query.contains_path(#toql_field_name))
                    } else {
                        quote!()
                    };

                    let role_test = if field.load_roles.is_empty() {
                        quote!()
                    } else {
                        let roles = &field.load_roles;
                        quote!(
                            toql::query::assert_roles(&self.roles, &[ #(String::from(#roles)),* ].iter().cloned().collect())
                            .map_err(|e| SqlBuilderError::RoleRequired(e))?;
                        )
                    };

                    let optional_join = if let Some(join) = &merge_attrs.join_sql {
                        if join.contains("..") {
                            let formatted_join = join.replace("..", "{alias}.");
                            quote!( result.push_join(&format!(#formatted_join, alias = &mapper.translated_alias(
                            &<#rust_type_ident as toql::sql_mapper::Mapped>::table_alias() ) )); )
                        } else {
                            quote!( result.push_join(#join);)
                        }
                    } else {
                        quote!()
                    };

                    let self_keys: Vec<&str> = merge_attrs
                        .columns
                        .iter()
                        .map(|c| c.this.as_str())
                        .collect();

                    // only validate user provided columns, auto generated column are always valid
                    let optional_self_column_validation = if merge_attrs.columns.is_empty() {
                         quote!()
                    } else {
                        quote!(
                            if cfg!(debug_assertions) {
                            for c in &[ #(#self_keys),* ] {
                                if !<#struct_ident as toql::key::Key>::columns().contains(&c.to_string()) {
                                let t = <#struct_ident as toql::sql_mapper::Mapped>::table_name();
                                let e = toql::sql_mapper::SqlMapperError::ColumnMissing(t, c.to_string());
                                let e2 = toql::error::ToqlError::SqlMapperError(e);
                                return Err(toql::mysql::error::ToqlMySqlError::ToqlError(e2));
                                }
                            }
                        }
                    )
                    };

                    // Custom joins may yield multiple records with same content
                    // In automatic joins this does not happen
                    let optional_distinct = if merge_attrs.join_sql.is_some() {
                        quote!(
                            dep_query.distinct = true; 
                        )
                    } else {
                        quote!()
                    };

                    // Column validation only, if no custom join is required
                    let optional_inverse_column_validation = if merge_attrs.join_sql.is_none() {
                        quote!(
                            if cfg!(debug_assertions) {
                            for c in &inverse_columns {
                                if !<#rust_type_ident as toql::key::Key>::columns().contains(c) {
                                let t = <#rust_type_ident as toql::sql_mapper::Mapped>::table_name();
                                let e = toql::sql_mapper::SqlMapperError::ColumnMissing(t, c.to_string());
                                let e2 = toql::error::ToqlError::SqlMapperError(e);
                                return Err(toql::mysql::error::ToqlMySqlError::ToqlError(e2));
                                }
                            }
                        }
                        )
                    } else {
                        quote!()
                    };

                    self.path_loaders.push( quote!(
                            #path_test {
                                #role_test
                                let table_name = <#rust_type_ident as toql::sql_mapper::Mapped>::table_name();
                                let mapper = cache.mappers.get(&table_name).ok_or(toql::error::ToqlError::MapperMissing(String::from(&table_name)))?;
                                let mut dep_query = query.clone();

                    #optional_self_column_validation

                    let default_inverse_columns= <#struct_ident as toql::key::Key>::default_inverse_columns();
                     let inverse_columns = <#struct_ident as toql::key::Key>::columns().iter().enumerate().map(|(i, c)| {

                        let inverse_column = match c.as_str() {
                                #(#inverse_column_translation)*
                            _ => {
                                    mapper.aliased_column(
                                        &<#rust_type_ident as toql::sql_mapper::Mapped>::table_alias(),
                                    default_inverse_columns.get(i).unwrap()
                                    )
                                }
                        };
                        inverse_column

                    }).collect::<Vec<String>>();

                    #optional_inverse_column_validation

                   // #predicate_builder
                    let (predicate, params) =
                            toql::key::predicate_from_columns_sql::<#struct_ident,_>(entity_keys, &inverse_columns);
                            dep_query.where_predicates.push(predicate);
                            dep_query.where_predicate_params.extend_from_slice(&params);
                           
                            #optional_distinct



                                let mut result =<Self as toql::load::Load<#rust_type_ident>>::build_path(self,#toql_field_name, &dep_query, cache)?;
                                if !result .is_empty() {

                                    // primary keys
                                     <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|c|{
                                        result.push_select(&mapper.aliased_column(&<#rust_type_ident as toql::sql_mapper::Mapped>::table_alias(),&c))
                                    });

                                    // foreign keys (inverse primary keys) on merged table
                                    inverse_columns.iter().for_each(|c|{
                                        result.push_select(&c);
                                    });


                                    #optional_join
                                    // foreign keys


                                    toql::log_sql!(result.to_sql(), result.params());
                                    let entities_stmt = self.conn().prep_exec(result.to_sql(), result.params())?;
                                    let (mut merge_entities, merge_keys, parent_keys)  = toql::mysql::row::from_query_result_with_merge_keys::<#rust_type_ident, <#rust_type_ident as toql::key::Key>::Key, <#struct_ident as toql::key::Key>::Key>(entities_stmt)?;
                                    
                                    if !merge_entities.is_empty() {
                                        self.load_dependencies(&mut merge_entities, &merge_keys, query, cache)?;

                                        toql::merge::merge(&mut entities, &entity_keys, merge_entities, &parent_keys,
                                            |e| { #merge_field_init;},
                                            |e,m|{ #merge_field_assign

                                                        }
                                                    )
                                    }
                                }

                            }
                        ));
                }
                _ => {
                    panic!("Should be never called!");
                }
            }
        }
    }
}

impl<'a> quote::ToTokens for GeneratedMysqlLoad<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
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

            fn from_row_with_index ( mut row : & mut toql::mysql::mysql :: Row , i : &mut usize)
                -> toql :: mysql :: error:: Result < #struct_ident> {

                Ok ( #struct_ident {
                    #(#mysql_deserialize_fields),*

                })
            }
            }


        );

        log::debug!(
            "Source code for `{}`:\n{}",
            &self.rust_struct.rust_struct_name,
            mysql.to_string()
        );

        tokens.extend(mysql);
    }
}
