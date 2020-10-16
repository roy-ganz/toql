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

pub(crate) struct GeneratedEntityFromRow<'a> {
    rust_struct: &'a Struct,

    mysql_deserialize_fields: Vec<TokenStream>,
    path_loaders: Vec<TokenStream>,
    ignored_paths: Vec<TokenStream>,

    forward_joins: Vec<TokenStream>,
    regular_fields: usize, // Impl for mysql::row::ColumnIndex,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
    merge_field_getter: HashMap<String, TokenStream>,
    wildcard_scope_code : TokenStream
}

impl<'a> GeneratedEntityFromRow<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedEntityFromRow {


        let wildcard_scope_code = if let Some(wildcard) = &toql.wildcard {

                let inserts = wildcard.iter()
                    .map(|w| quote!(
                            scopes.insert(#w .to_string());
                            )
                        )
                    .collect::<Vec<_>>();

              quote!(
                  let mut scopes : std::collections::HashSet<String> = std::collections::HashSet::new();
                   #(#inserts)*                
                  let wildcard_scope = toql::sql_builder::wildcard_scope::WildcardScope::Only(scopes);
              )
        } else {
            quote!( let wildcard_scope = toql::sql_builder::wildcard_scope::WildcardScope::All; )
        };

        GeneratedEntityFromRow {
            rust_struct: &toql,
            mysql_deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            forward_joins: Vec::new(),
            regular_fields: 0,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
            merge_field_getter: HashMap::new(),
            wildcard_scope_code
        }
    }

    pub(crate) fn add_mysql_deserialize_skip_field(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;

        self.mysql_deserialize_fields.push( quote!( #rust_field_ident : Default::default()));
              
              
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

                    // Check selection for optional Toql fields: Option<Option<..> or Option<..>
                    if field.number_of_options > 0 {
                        self.mysql_deserialize_fields.push(quote!(
                            #rust_field_ident : {
                                if iter.next().unwrap_or(&Select::None) != &Select::None {
                                   
                                    row.get_opt( (*i,  *i += 1).0).unwrap()
                                        .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                                } else {
                                    None
                                }
                            }
                        ));
                    } 
                    // Preselected fields
                    else {
                        self.mysql_deserialize_fields.push(quote!(
                            #rust_field_ident : {
                                if iter.next().unwrap_or(&Select::None) == &Select::None{
                                     return Err(toql::error::ToqlError::DeserializeError(#error_field.to_string(), String::from("Deserialization stream is invalid: Expected selected field but got unselected.")).into());
                                }
                              
                                row.get_opt((*i,  *i += 1).0).unwrap()
                                    .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                            }
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
                   /*  self.forward_joins
                        .push(
                            
                            if  field.number_of_options  == 2 {
                                // Skip discriminator field from selectable left joins (Option<Option<T>>)
                                quote!( i = < #rust_type_ident > ::forward_row(i) + 1;)
                            } else {
                                quote!( i = < #rust_type_ident > ::forward_row(i);)
                            }); */
            



                    // For optional joined fields (left Joins) a discriminator field must be added to check
                    // - for unselected entity (discriminator column is NULL Type)
                    // - for null entity (discriminator column is false) - only left joins

                    self.mysql_deserialize_fields.push(
                    match   field.number_of_options {
                        2 =>   //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
                        quote!(
                                    #rust_field_ident : {
                                        
                                          if iter.next().unwrap_or(&Select::None) != &Select::None {
                                            if row.get_opt::<bool,_>(*i).unwrap()
                                                .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                                                == false {
                                                *i += 1;  // Step over discriminator field
                                                Some(None)
                                            } else {
                                                *i += 1; // Step over discriminator field
                                                Some(Some(< #rust_type_ident > :: from_row_with_index ( & mut row , i, iter )?))
                                            }
                                        } else {
                                            None
                                        }
                                    }
                            ),
                        1 if field.preselect =>   //    #[toql(preselect)] Option<T>  -> Nullable Join -> Left Join
                                quote!(
                                    #rust_field_ident : {
                                        if iter.next().unwrap_or(&Select::None) == &Select::None {
                                            return Err(toql::error::ToqlError::DeserializeError(#error_field.to_string(), String::from("Deserialization stream is invalid: Expected selected field but got unselected.")).into());
                                        }
                                        if row.get_opt::<bool,_>(*i).unwrap()
                                        .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?
                                        == false {
                                            None
                                        } else {
                                            Some(< #rust_type_ident > :: from_row_with_index ( & mut row , i, iter )?)
                                        }
                                    }
                                ),
                            1 if !field.preselect =>  //    Option<T>                         -> Selectable Join -> Inner Join
                                        quote!(
                                        #rust_field_ident : {
                                        
                                            if  iter.next().unwrap_or(&Select::None) == &Select::None {
                                                None
                                            } else {
                                            Some(< #rust_type_ident > :: from_row_with_index ( & mut row , i, iter )?)
                                            }
                                        }
                                    ),
                        _ =>   //    T                                 -> Selected Join -> InnerJoin
                        quote!(
                            #rust_field_ident : { 
                                 if iter.next().unwrap_or(&Select::None) == &Select::None {
                                     return Err(toql::error::ToqlError::DeserializeError(#error_field.to_string(), String::from("Deserialization stream is invalid: Expected selected field but got unselected.")).into());
                                }
                                < #rust_type_ident > :: from_row_with_index ( & mut row , i, iter )?}
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
            )
        } else {
            quote!(
                fn load_dependencies(&mut self, mut entities: &mut Vec< #struct_ident>,mut entity_keys: & Vec< #struct_key_ident >,
                query: &toql::query::Query<#struct_ident>, 
                wildcard_scope: &toql::sql_builder::wildcard_scope::WildcardScope,
             )
                -> Result<(), toql::mysql::error::ToqlMySqlError>
                {
                  
                    if entities.is_empty() {
                        return Ok(());
                    }
                    #(#path_loaders)*
                    Ok(())
                }
            )
        };

        let optional_add_primary_keys = if self.merge_fields.is_empty() {
            quote!( Vec::new())
        } else {
            quote!(
                 <<#struct_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().map(|c|{
                    mapper.aliased_column(&<#struct_ident as toql::sql_mapper::mapped::Mapped>::table_alias(),&c)
                    }).collect::<Vec<_>>()
                 
            )
        };

        let from_query_result = if self.merge_fields.is_empty() {
            quote!(
                let mut entities = toql::mysql::row::from_query_result::< #struct_ident, _ >(entities_stmt, selection_stream)?;
            )
        } else {
            quote!(
            let (mut entities, keys) = toql::mysql::row::from_query_result_with_primary_keys::<#struct_ident,_, #struct_key_ident>(entities_stmt, selection_stream)?;
            )
        };

        let optional_load_merges = if self.merge_fields.is_empty() {
            quote!()
        } else {
            quote!(
            self.load_dependencies(&mut entities, &keys, &query, &wildcard_scope)?;
            )
        };
        

        let wildcard_scope_code = &self.wildcard_scope_code;
        quote!(

            impl<'a, T: toql::mysql::mysql::prelude::GenericConnection + 'a> toql::load::Load<#struct_ident> for toql::mysql::MySql<'a,T>
            {
                type Error = toql :: mysql::error::ToqlMySqlError;

                fn load_one(&mut self, query: &toql::query::Query<#struct_ident>)
                    -> Result<# struct_ident, toql :: mysql::error:: ToqlMySqlError>

                {
                 
                   // let mapper = self.registry().mappers.get( #struct_name).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;

              //      #wildcard_scope_code

                  let columns :Vec<String> = #optional_add_primary_keys;

                    let result = toql::sql_builder::SqlBuilder::new( #struct_name, self.registry())
          //               #(#ignored_paths)*
                        .build_select_result("", &query, self.alias_format())?;
                    
                    let sql = result.select_sql_with_additional_columns("", "LIMIT 0, 2",&columns);
                    let selection_stream = result.selection_stream();

                     toql::log_sql!(&sql);

                    let args = toql::mysql::sql_arg::values_from_ref(&sql.1);
                    let entities_stmt = self.conn().prep_exec(sql.0, args)?;

                    #from_query_result

                    if entities.len() > 1 {
                        return Err(toql::mysql::error::ToqlMySqlError::ToqlError(toql::error::ToqlError::NotUnique));
                    } else if entities.is_empty() {
                        return Err(toql::mysql::error::ToqlMySqlError::ToqlError(toql::error::ToqlError::NotFound));
                    }

                    #optional_load_merges
                    Ok(entities.pop().unwrap())
                }


                fn load_many( &mut self, query: &toql::query::Query<#struct_ident>,

                
                page: Option<toql::load::Page>)
                -> Result<(std::vec::Vec< #struct_ident >, Option<(u32, u32)>), toql :: mysql::error:: ToqlMySqlError>
                {
                   

                    let mut count = false;
                 
                    let sql_page : String= match page {
                        None => String::from(""),
                        Some(toql::load::Page::Counted(f, m)) =>  {count = true; format!("LIMIT {}, {}", f, m)},
                        Some(toql::load::Page::Uncounted(f, m)) =>  {count = false;format!("LIMIT {}, {}", f, m)},
                    };

                    #wildcard_scope_code

                 /*    let mapper = self.registry().mappers.get( #struct_name)
                            .ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?; */
                    // load base entities

                    let hint = String::from( if count {"SQL_CALC_FOUND_ROWS" }else{""});

                 
                    let columns :Vec<String>= #optional_add_primary_keys;

                    let result = toql::sql_builder::SqlBuilder::new(#struct_name, self.registry())
                    .with_roles( self.roles().clone())
                      //   #(#ignored_paths)*
                      //  .scope_wildcard(&wildcard_scope)
                        .build_select_result("", &query, self.alias_format())?;
          
                        let sql = result.select_sql_with_additional_columns( &hint, &sql_page, &columns);
                        let selection_stream = result.selection_stream();
       
                    toql::log_sql!(&sql);


                   // toql::log_sql!(toql::mysql::sql_from_query_result( &result, &hint, sql_page.to_owned()), result.query_params());
                    let args = toql::mysql::sql_arg::values_from_ref(&sql.1);
                    let entities_stmt = self.conn().prep_exec(sql.0, args)?;
                    #from_query_result
                    
                    let mut count_result = None;

                    // Get count values
                    if count {
                                let total_count = {
                                     toql::log_sql!("SELECT FOUND_ROWS();", Vec::<String>::new());
                                    let r = self.conn().query("SELECT FOUND_ROWS();")?;
                                    r.into_iter().next().unwrap().unwrap().get(0).unwrap()
                                };
                                let filtered_count = {
                            /*   let mapper = self.registry() // Get new mapper because slef is mut borrowed by self.conn()
                                        .mappers
                                        .get( #struct_name)
                                        .ok_or(toql::error::ToqlError::MapperMissing(String::from("User")))?; */


                                    let sql = toql::sql_builder::SqlBuilder::new( #struct_name, self.registry())
                                    .with_roles(self.roles().clone())
                                    //#(#ignored_paths)*
                                    .build_count_sql("", &query, "", "", self.alias_format())?;
                            
                                    toql::log_sql!( sql);
                                    
                                    let args = toql::mysql::sql_arg::values_from_ref(&sql.1);
                                    let r = self.conn().prep_exec( sql.0, args)?;
                                    r.into_iter().next().unwrap().unwrap().get(0).unwrap()
                                };
                                count_result = Some((total_count ,filtered_count))
                    }

                   #optional_load_merges

                    Ok((entities, count_result))
                }
                /* fn build_path(&mut self,path: &str, query: &toql::query::Query<#struct_ident>,
                    wildcard_scope: &toql::sql_builder::wildcard_scope::WildcardScope,
                    additional_columns: &[String]
                   )
                -> Result<Option<toql::sql::Sql>,toql :: mysql::error:: ToqlMySqlError>
                {
                     // Get new mapper, because self is mut borrowed 
                  //  let mapper = self.registry().mappers.get( #struct_name ).ok_or( toql::error::ToqlError::MapperMissing(String::from(#struct_name)))?;
                   let result = toql::sql_builder::SqlBuilder::new( #struct_name, self.registry())
                    .with_roles(self.roles().clone())
                       // .scope_wildcard(&wildcard_scope)
                        .build_select_result(path, &query, self.alias_format.clone())?;
                     let sql = result.select_sql_with_additional_columns("", "", &additional_columns);

                     Ok(Some(sql))
                        
                } */

                #load_dependencies_from_mysql
            }

        )
    }

    pub fn build_merge(&mut self) {
        // Build all merge fields
        // This must be done after the first pass, becuase all key names must be known at this point
        let struct_ident = &self.rust_struct.rust_struct_ident;
        
        for field in &self.merge_fields {
            let rust_type_ident = &field.rust_type_ident;
            let rust_type_name = &field.rust_type_name;
            let rust_field_ident = &field.rust_field_ident;
            let toql_field_name = &field.toql_field_name;

            match &field.kind {
                FieldKind::Merge(merge_attrs) => {
                    let mut inverse_column_translation: Vec<TokenStream> = Vec::new();

                    for m in &merge_attrs.columns {
                        let untranslated_column = &m.this;
                        match &m.other {
                            MergeColumn::Unaliased(other_column) => {
                                inverse_column_translation.push( quote!( #untranslated_column => mapper.aliased_column(&<#rust_type_ident as toql::sql_mapper::mapped::Mapped>::table_alias(),#other_column), ));
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
                        if field.skip_wildcard {
                               quote!( if query.contains_path_starts_with(#toql_field_name) && wildcard_scope.contains_path(#toql_field_name) )
                        }else {
                            quote!( if query.contains_path(#toql_field_name) && wildcard_scope.contains_path(#toql_field_name) )
                        }
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
                    
                   

                    let custom_join_code = if let Some(join) = &merge_attrs.join_sql {

                         // Quick guess if params are used
                        let optional_join_code = if join.contains('<')
                        {
                            quote!(
                            let ( join_stmt, join_params) = toql::sql_builder::SqlBuilder::resolve_query_params(join_stmt, &query.aux_params)
                                .map_err(|e| toql::mysql::error::ToqlMySqlError::ToqlError( toql::error::ToqlError::SqlBuilderError(e)))?;
                                dep_query.join_stmt_params.extend_from_slice(&join_params);
                                dep_query.join_stmts.push(join_stmt);
                            )
                        } else {
                            quote!(dep_query.join_stmts.push(join_stmt.to_string());)
                        };
                       
                       

                       let aliased_join_statement =  if join.contains("...") {
                            let formatted_join = join.replace("...", "{alias}.");
                            quote!( 
                                let join_stmt = &format!(#formatted_join, alias = &mapper.translated_alias(
                                &<#rust_type_ident as toql::sql_mapper::mapped::Mapped>::table_alias() ));
                            )
                        } else {
                            quote!( let join_stmt = #join; )
                        };
                       
                        quote!(
                            #aliased_join_statement
                            #optional_join_code
                        )
                    } else {
                        quote!()
                    };

                    let self_keys: Vec<&str> = merge_attrs
                        .columns
                        .iter()
                        .map(|c| c.this.as_str())
                        .collect();

                    // Only validate user provided columns, auto generated column are always valid
                    let optional_self_column_validation = if merge_attrs.columns.is_empty() {
                         quote!()
                    } else {
                         quote!(
                            if cfg!(debug_assertions) {
                            for c in &[ #(#self_keys),* ] {

                                if ! <<#struct_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().contains(&c.to_string()) {
                                let t = <#struct_ident as toql::sql_mapper::mapped::Mapped>::table_name();
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
                    // Does not work, because inverse columns are not keys
                   /*  let optional_inverse_column_validation = if merge_attrs.join_sql.is_none() {
                        quote!(
                            if cfg!(debug_assertions) {
                            for c in &inverse_columns {
                                 let unaliased_column =  if let Some(i) = c.find('.') {
                                        String::from(&c[(i + 1)..])
                                      } else {
                                        String::from(c)
                                    };
                                if !<#rust_type_ident as toql::key::Keyed>::columns().contains(&unaliased_column) {
                                let t = <#rust_type_ident as toql::sql_mapper::mapped::Mapped>::table_name();
                                let e = toql::sql_mapper::SqlMapperError::ColumnMissing(t, unaliased_column);
                                let e2 = toql::error::ToqlError::SqlMapperError(e);
                                return Err(toql::mysql::error::ToqlMySqlError::ToqlError(e2));
                                }
                            }
                        }
                        )
                    } else {
                        quote!()
                    }; */

                    self.path_loaders.push( quote!(
                            #path_test {
                                #role_test
                                let type_name = <#rust_type_ident as toql::sql_mapper::mapped::Mapped>::type_name();
                                let mapper = self.registry().mappers.get(&type_name).ok_or(toql::error::ToqlError::MapperMissing(String::from(&type_name)))?;
                                let mut dep_query = query.traverse::<#rust_type_ident >(#toql_field_name); //clone_for_type();

                    #optional_self_column_validation

                    let default_inverse_columns= <<#struct_ident as toql::key::Keyed>::Key as toql::key::Key>::default_inverse_columns();
                     let inverse_columns = <<#struct_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().enumerate().map(|(i, c)| {

                        let inverse_column = match c.as_str() {
                                #(#inverse_column_translation)*
                            _ => {
                                    mapper.aliased_column(
                                        &<#rust_type_ident as toql::sql_mapper::mapped::Mapped>::table_alias(),
                                    default_inverse_columns.get(i).unwrap()
                                    )
                                }
                        };
                        inverse_column

                    }).collect::<Vec<String>>();

                   // #optional_inverse_column_validation

                   // #predicate_builder
                    let (predicate, params) =
                            toql::key::predicate_from_columns_sql::<< #struct_ident as toql::key::Keyed>::Key,_>(entity_keys, &inverse_columns);
                            dep_query.where_predicates.push(predicate);
                            dep_query.where_predicate_params.extend_from_slice(&params);

                            #custom_join_code
                           
                            #optional_distinct


                                 // primary keys
                                let mut columns :Vec<String>=  <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().map(|c|{
                                        mapper.aliased_column(&<#rust_type_ident as toql::sql_mapper::mapped::Mapped>::table_alias(),&c)
                                }).collect::<Vec<_>>();

                                columns.extend_from_slice(&inverse_columns);
                                let mut result =<Self as toql::load::Load<#rust_type_ident>>::build_path(self,#toql_field_name, &dep_query, &wildcard_scope, &columns)?;
                                if let Some(sql) = result {

                                    let mapper = self.registry().mappers.get(#rust_type_name).ok_or(toql::error::ToqlError::MapperMissing(String::from(#rust_type_name)))?;

                                   

                                 
                                    // foreign keys


                                    toql::log_sql!(sql.0, sql.1);
                                    let args = toql::mysql::sql_arg::values_from_ref(&sql.1);
                                    let entities_stmt = self.conn().prep_exec(sql.0, args)?;
                                    let (mut merge_entities, merge_keys, parent_keys)  = toql::mysql::row::from_query_result_with_merge_keys::<#rust_type_ident, _, <#rust_type_ident as toql::key::Keyed>::Key, <#struct_ident as toql::key::Keyed>::Key>(entities_stmt)?;
                                    
                                    if !merge_entities.is_empty() {
                                        dep_query.join_stmt_params.clear();
                                        dep_query.join_stmts.clear();
                                        dep_query.where_predicate_params.clear();
                                        dep_query.where_predicates.clear(); 
                                        let dep_query =  dep_query.traverse::<#rust_type_ident>(#toql_field_name);

                                        self.load_dependencies(&mut merge_entities, &merge_keys, &dep_query, &wildcard_scope)?; // dep_query anstatt query
                                    }
                                    toql::merge::merge(&mut entities, &entity_keys, merge_entities, &parent_keys,
                                        |e| { #merge_field_init;},
                                        |e,m|{ #merge_field_assign

                                                    }
                                                );
                                    
                                } else {
                                    entities.iter_mut().for_each(|mut e| { #merge_field_init;});
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

impl<'a> quote::ToTokens for GeneratedEntityFromRow<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;
        let loader = self.loader_functions();

        let mysql_deserialize_fields = &self.mysql_deserialize_fields;

        let regular_fields = self.regular_fields;
        let forward_joins = &self.forward_joins;

        let macro_name = Ident::new(&format!("toql_entity_from_row_{}", &struct_ident), Span::call_site());
     
        let mysql = quote!(

         //   #loader


           // impl toql :: mysql :: row:: FromResultRow < #struct_ident > for #struct_ident {

            macro_rules! #macro_name {
                        ($row_type: ty, $col_get: ident) => {

            impl toql::from_row::FromRow<toql::mysql::mysql::Row> for #struct_ident {
 
             type Error = toql::mysql::error::ToqlMySqlError;
            /*  fn skip(mut i : usize) -> usize {
                i += #regular_fields ;
                #(#forward_joins)*
                i
            }   */

            fn from_row_with_index<'a, I> ( mut row : &mysql::Row , i : &mut usize, mut iter: &mut I)
                -> toql :: mysql :: error:: Result < #struct_ident> 
                where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> {

                    use toql::sql_builder::select_stream::Select;


                            
      /*   let row : & mysql :: Row = row . as_ref() 
            .map_err(| e | {  toql::error::ToqlError::DeserializeError(#struct_name.to_owned(), e.to_string())})? ; */

       
                Ok ( #struct_ident {
                    #(#mysql_deserialize_fields),*

                })
            }
            }
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
