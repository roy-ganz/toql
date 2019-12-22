use quote::quote;

use crate::sane::{FieldKind, SqlTarget, Struct};

use heck::SnakeCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::Ident;

pub(crate) struct GeneratedToqlMapper<'a> {
    rust_struct: &'a Struct,
    merge_functions: Vec<TokenStream>,
    field_mappings: Vec<TokenStream>,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
}

impl<'a> GeneratedToqlMapper<'a> {
    pub(crate) fn from_toql(rust_struct: &'a Struct) -> GeneratedToqlMapper {
        
        let mut field_mappings : Vec<TokenStream> = Vec::new();
        for mapping in &rust_struct.mapped_fields {
            let toql_field_name = &mapping.field;
            let sql_mapping = &mapping.sql;

            match &mapping.handler {
                    Some(handler) => {
                        field_mappings.push(quote! {
                                mapper.map_handler_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                #sql_mapping, #handler (), toql::sql_mapper::FieldOptions::new());
                            });
                    },
                    None => {
                        field_mappings.push(quote! {
                                mapper.map_field_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                #sql_mapping,toql::sql_mapper::FieldOptions::new());
                            });
                    }
            }
        }

        GeneratedToqlMapper {
            rust_struct,
            merge_functions: Vec::new(),
            field_mappings,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
        }

    }

    pub(crate) fn add_field_mapping(&mut self, field: &crate::sane::Field) -> Result<(), ()> {
        let rust_field_name = &field.rust_field_name;

        // Joined field
        match &field.kind {
            FieldKind::Join(join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;
                let rust_type_ident = &field.rust_type_ident;
                let toql_field_name = &field.toql_field_name;
                let join_alias = &join_attrs.join_alias;
                let sql_join_table_name = &join_attrs.sql_join_table_name;

                // Add discriminator field for LEFT joins
                if field.number_of_options > 1 || (field.number_of_options == 1 && field.preselect == true) {
                    self.field_mappings.push(
                                    quote!(
                                        let none_condition = <#rust_type_ident as toql::key::Key>::columns().iter().map(|other_column|{
                                                #default_self_column_code;
                                                let self_column = #columns_map_code;
                                                format!("({}{}{} IS NOT NULL)",sql_alias, if sql_alias.is_empty() { "" } else { "." }, self_column)
                                        }).collect::<Vec<String>>().join(" AND ");   
                                        mapper.map_field_with_options(
                                        &format!("{}{}{}_",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), &none_condition,toql::sql_mapper::FieldOptions::new().preselect(true));
                                )
                                );
                }

                let join_expression_builder = quote!(
                  let join_expression = <#rust_type_ident as toql::key::Key>::columns().iter()
                    .map(|other_column| {
                        #default_self_column_code;
                        let self_column= #columns_map_code;
                        format!("{}{}{} = {}.{}",sql_alias , if sql_alias.is_empty() { "" } else { "." }, self_column, #join_alias, other_column)
                    }).collect::<Vec<String>>().join(" AND ")
                );

                let on_sql = if let Some(ref sql) = &join_attrs.on_sql {
                    format!(" AND ({})", sql.replace("..", &format!("{}.", join_alias)))
                } else {
                    String::from("")
                };

             
               
                let join_aliased_table = format!("{} {}",  sql_join_table_name, join_alias);
                let join_predicate_format = format!("{{}}{}",on_sql); 
                let join_predicate = quote!(&format!( #join_predicate_format, join_expression));

                let join_type = if field.number_of_options == 0 || (field.number_of_options == 1 && field.preselect == false) {
                    quote!(toql::sql_mapper::JoinType::Inner)   
                } else {
                    quote!(toql::sql_mapper::JoinType::Left)   
                };

                  let select_ident = if field.preselect || (field.number_of_options == 0) {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };
                let ignore_wc_ident = if field.skip_wildcard {
                    quote!( .ignore_wildcard(true))
                } else {
                    quote!()
                };

                let roles = &field.roles;
                let roles_ident = if roles.is_empty() {
                    quote!()
                } else {
                    quote! { .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  }
                };
                
                self.field_mappings.push(quote! {
                    #join_expression_builder;
                    mapper.map_join::<#rust_type_ident>( &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), #join_alias);
                    mapper.join_with_options( &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                     #join_type,  
                     #join_aliased_table, 
                     #join_predicate,
                     toql::sql_mapper::JoinOptions::new() #select_ident #ignore_wc_ident #roles_ident );
                });

                if join_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Regular(regular_attrs) => {
                let toql_field_name = &field.toql_field_name;
                let countfilter_ident = if regular_attrs.count_filter {
                    quote!( .count_filter(true))
                } else {
                    quote!()
                };
                let countselect_ident = if regular_attrs.count_select {
                    quote!( .count_select(true))
                } else {
                    quote!()
                };
                let select_ident = if field.preselect || (field.number_of_options == 0) {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };
                let ignore_wc_ident = if field.skip_wildcard {
                    quote!( .ignore_wildcard(true))
                } else {
                    quote!()
                };

                let roles = &field.roles;
                let roles_ident = if roles.is_empty() {
                    quote!()
                } else {
                    quote! { .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  }
                };

                let sql_mapping = match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        quote! {&format!("({})", #expression .replace("..",&format!("{}.",sql_alias )))}
                    }
                    SqlTarget::Column(ref column) => {
                        quote! {&format!("{}{}{}",sql_alias, if sql_alias.is_empty() {"" }else {"."}, #column)}
                    }
                };

                match &regular_attrs.handler {
                    Some(handler) => {
                        self.field_mappings.push(quote! {
                                            mapper.map_handler_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                            #sql_mapping, #handler (), toql::sql_mapper::FieldOptions::new() #select_ident #countfilter_ident #countselect_ident #ignore_wc_ident #roles_ident);
                                        }
                            );
                    }
                    None => {
                        self.field_mappings.push(quote! {
                                            mapper.map_field_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                            #sql_mapping,toql::sql_mapper::FieldOptions::new() #select_ident #countfilter_ident #countselect_ident #ignore_wc_ident #roles_ident);
                                        }
                            );
                    }
                };

                if regular_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Merge(ref _merge_attrs) => {}
        };
        Ok(())
    }

    pub(crate) fn add_merge_function(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.to_owned());
    }

    pub fn build_merge(&mut self) {
        // Build all merge fields
        // This must be done after the first pass, becuase all key names must be known at this point
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;
        for field in &self.merge_fields {
            let rust_type_ident = &field.rust_type_ident;
            let rust_field_ident = &field.rust_field_ident;
            

            match &field.kind {
                FieldKind::Merge(merge_attrs) => {
                    let function_ident =
                        syn::Ident::new(&format!("merge_{}", rust_field_ident), Span::call_site());

                    let mut this_tuple: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut other_tuple: Vec<proc_macro2::TokenStream> = Vec::new();
                    for this_field in &self.key_field_names {
                        let default_other_field =
                            format!("{}_{}", struct_name.to_snake_case(), &this_field);
                        let other_field = merge_attrs.other_field(&this_field, default_other_field);

                        let this_key_field = Ident::new(&this_field, Span::call_site());
                        this_tuple.push(quote!(t. #this_key_field));
                        let other_key_field = Ident::new(&other_field, Span::call_site());
                        other_tuple.push(quote!(o. #other_key_field));
                    }

                    let this_tuple_ref = &this_tuple;
                    let other_tuple_ref = &other_tuple;

                    let self_fnc: proc_macro2::TokenStream = if self.key_field_names.len() == 1 {
                        quote!( Option::from( #(#this_tuple_ref)*) )
                    } else {
                        quote!( if #( (Option::from (#this_tuple_ref)).or)* (None).is_some() { Option::from((#(#this_tuple_ref),* ))} else {None} )
                    };
                    let other_fnc: proc_macro2::TokenStream = if self.key_field_names.len() == 1 {
                        quote!( Option::from( #(#other_tuple_ref)*) )
                    } else {
                        quote!( if #( (Option::from (#other_tuple_ref)).or)* (None).is_some() { Option::from((#(#other_tuple_ref),* ))} else {None} )
                    };

                    self.merge_functions.push(quote!(
                                    pub fn #function_ident ( t : & mut Vec < #struct_ident > , o : Vec < #rust_type_ident > ) {
                                            toql :: merge :: merge ( t , o ,
                                            | t | #self_fnc ,
                                            | o | #other_fnc ,
                                            | t , o |  {let t : Option<&mut Vec<#rust_type_ident>>= Option::from( &mut t. #rust_field_ident ); if t.is_some() { t.unwrap().push(o);}}
                                            ) ;
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

impl<'a> quote::ToTokens for GeneratedToqlMapper<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;

        let sql_table_name = &self.rust_struct.sql_table_name;
        let sql_table_alias = &self.rust_struct.sql_table_alias;

        let merge_functions = &self.merge_functions;

        let field_mappings = &self.field_mappings;

        let builder = quote!(

            impl toql::sql_mapper::Mapped for #struct_ident {

                fn table_name() -> String {
                    String::from(#sql_table_name)
                }
                fn table_alias() -> String {
                    String::from(#sql_table_alias)
                }
                fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str, sql_alias: &str) {
                    #(#field_mappings)*
                }
            }

            impl #struct_ident {

                #(#merge_functions)*

            }

        );

        log::debug!(
            "Source code for `{}`:\n{}",
            &self.rust_struct.rust_struct_ident,
            builder.to_string()
        );

        tokens.extend(builder);
    }
}
