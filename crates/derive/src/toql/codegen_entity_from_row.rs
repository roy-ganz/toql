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

pub(crate) struct CodegenEntityFromRow<'a> {
    rust_struct: &'a Struct,

    deserialize_fields: Vec<TokenStream>,
    path_loaders: Vec<TokenStream>,
    ignored_paths: Vec<TokenStream>,

    
    regular_fields: usize, 
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
    merge_field_getter: HashMap<String, TokenStream>,
    
}

impl<'a> CodegenEntityFromRow<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenEntityFromRow {

        CodegenEntityFromRow {
            rust_struct: &toql,
            deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            regular_fields: 0,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
            merge_field_getter: HashMap::new(),
        }
    }

    pub(crate) fn add_deserialize_skip_field(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;

        self.deserialize_fields.push( quote!( #rust_field_ident : Default::default()));
              
              
    }


    pub(crate) fn add_deserialize(&mut self, field: &crate::sane::Field) {
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
                        self.deserialize_fields.push(quote!(
                            #rust_field_ident : {
                                if iter.next().unwrap_or(&Select::None) != &Select::None {
                                    ($col_get!(row, *i)
                                        .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?,
                                    *i += 1).0
                                } else {
                                    None
                                }
                            }
                        ));
                    } 
                    // Preselected fields
                    else {
                        self.deserialize_fields.push(quote!(
                            #rust_field_ident : {
                                if iter.next().unwrap_or(&Select::None) == &Select::None{
                                     return Err(toql::error::ToqlError::DeserializeError(#error_field.to_string(), String::from("Deserialization stream is invalid: Expected selected field but got unselected.")).into());
                                }
                              
                               ($col_get!(row, *i)
                                    .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?,
                                *i += 1).0
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

                    self.deserialize_fields.push(
                    match   field.number_of_options {
                        2 =>   //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
                        quote!(
                                    #rust_field_ident : {
                                        
                                          if iter.next().unwrap_or(&Select::None) != &Select::None {
                                              let j : bool = $col_get!(row, *i)
                                                .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?;
                                            if j == false {
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
                                        let j : bool = $col_get!(row, *i)
                                        .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string()))?;
                                       if j  == false {
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
                    self.deserialize_fields
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
 
                    let role_test = match &field.roles.load {
                        Some(role) => {  quote!(
                            if !toql::role_validator::RoleValidator::is_valid(toql::role_expr_parser::RoleExprParser(#role)?) {
                                SqlBuilderError::RoleRequired(#role)
                            }
                            
                        )},
                        None => quote!()
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

impl<'a> quote::ToTokens for CodegenEntityFromRow<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
    

        let deserialize_fields = &self.deserialize_fields;

    
       
     
        let code = quote!(

         //   #loader


           // impl toql :: mysql :: row:: FromResultRow < #struct_ident > for #struct_ident {

            impl<R,E> toql::from_row::FromRow<R, E> for #struct_ident {
 
           
            #[allow(unused_variables, unused_mut)]
            fn from_row_with_index<'a, I> ( mut row : &R , i : &mut usize, mut iter: &mut I)
                ->std::result:: Result < #struct_ident, E> 
                where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> {

                    use toql::sql_builder::select_stream::Select;


                            
    

       
                Ok ( #struct_ident {
                    #(#deserialize_fields),*

                })
            }
            }
           

        );

        log::debug!(
            "Source code for `{}`:\n{}",
            &self.rust_struct.rust_struct_name,
            code.to_string()
        );

        tokens.extend(code);
    }
}
