use crate::annot::Toql;
use crate::annot::ToqlField;
use quote::quote;

/* use proc_macro2::Span;

use heck::MixedCase;
use heck::SnakeCase;
use syn::Ident;
 */
use crate::sane::{Field, FieldKind, JoinField, MergeField, RegularField, SqlTarget, Struct};
use heck::MixedCase;
use heck::SnakeCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::Ident;

pub(crate) struct GeneratedToqlMapper<'a> {
    rust_struct: &'a Struct,
    /* struct_ident: &'a Ident,

    sql_table_name: String,
    sql_table_alias: String,   */
    merge_functions: Vec<TokenStream>,
    field_mappings: Vec<TokenStream>,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
}

impl<'a> GeneratedToqlMapper<'a> {
    pub(crate) fn from_toql(rust_struct: &'a Struct) -> GeneratedToqlMapper {
        // let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        GeneratedToqlMapper {
            rust_struct,

            /* sql_table_name: rust_struct.sql_table_name,
            sql_table_alias: rust_struct.sql_table_alias, */
            merge_functions: Vec::new(),
            field_mappings: Vec::new(),
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
        }
    }

    pub(crate) fn add_field_mapping(&mut self, field: &crate::sane::Field) -> Result<(), ()> {
        /*   let field_ident = &field.ident.as_ref().unwrap();

        let toql_field = format!("{}", field_ident).to_mixed_case();

        let renamed_sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

        let sql_field: &str = match &field.column {
            Some(string) => string,
            None => &renamed_sql_column,
        }; */
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

                if field.number_of_options > 0 {
                    self.field_mappings.push(
                                    quote!(
                                        let none_condition = <#rust_type_ident as toql::key::Key>::columns().iter().map(|other_column|{
                                                #default_self_column_code;
                                                let self_column = #columns_map_code;
                                                format!("({}{}{} IS NOT NULL)",sql_alias, if sql_alias.is_empty() { "" } else { "." }, self_column)
                                        }).collect::<Vec<String>>().join(" AND ");   
                                        mapper.map_field_with_options(
                                        &format!("{}_", #toql_field_name), &none_condition,toql::sql_mapper::MapperOptions::new().preselect(true));
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

                let format_string = format!(
                    "{}JOIN {} {} ON ({{}}{})",
                    if field.number_of_options == 2
                        || (field.number_of_options == 1 && field.preselect == true)
                    {
                        "LEFT "
                    } else {
                        ""
                    },
                    sql_join_table_name,
                    join_alias,
                    on_sql
                );

                let join_clause = quote!(&format!( #format_string, join_expression));
                let join_selected = field.number_of_options == 0
                    || (field.number_of_options == 1 && field.preselect == true);
                self.field_mappings.push(quote! {
                    #join_expression_builder;
                    mapper.map_join::<#rust_type_ident>(  #toql_field_name, #join_alias);
                    mapper.join( #toql_field_name, #join_clause, #join_selected );
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
                let ignore_wc_ident = if field.ignore_wildcard {
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

                self.field_mappings.push(quote! {
                                            mapper.map_field_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                            #sql_mapping,toql::sql_mapper::MapperOptions::new() #select_ident #countfilter_ident #countselect_ident #ignore_wc_ident #roles_ident);
                                        }
                            );

                if regular_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Merge(ref merge_attrs) => {}
        };

        /* if field.join.is_some() {
                    // let renamed_join_column = crate::util::rename_sql_column(&field_ident.to_string(),&toql.columns);

                    let joined_struct_ident = field.first_non_generic_type();
                    let joined_struct_name = field.first_non_generic_type().unwrap().to_string();
                    let default_join_alias = joined_struct_name.to_snake_case();
                    let renamed_join_table = crate::util::rename(&joined_struct_name, &toql.tables);
                    let join_table = &field.table.as_ref().unwrap_or(&renamed_join_table);
                    let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);


                  /*   // default self columns
                    let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_ident), &toql.columns)];
                    let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() {
                        field.join.as_ref().unwrap().this_columns.as_ref() }
                        else {
                            &default_self_columns
                        }; */
                   // let self_columns : &Vec<String>= field.join.as_ref().unwrap().this_columns.as_ref();


                    // Map field with None condition
                    // If this condition is true, the following joined entity is NONE
                    if field.number_of_options() > 0 {
                        /* let none_condition: String = self_columns
                            .iter()
                            .map(|self_column| {

                                //let this_key = j.this_column.as_ref().unwrap_or(&auto_self_key);
                                format!("({{alias}}{{sep}}{} IS NOT NULL)", self_column)
                            })
                            .collect::<Vec<String>>()
                            .join(" AND "); */

                     let default_column_format = format!("{}_{{}}", field_ident);
                     let match_translation  = field.join.as_ref().unwrap().columns.iter()
                                .map(|column| {
                                    let tc = &column.this; let oc = &column.other;
                                    quote!( #oc => #tc,)
                                })
                                .collect::<Vec<_>>();

                        self.field_mappings.push(
                            quote!(
                                let none_condition = <#joined_struct_ident as toql::key::Key>::columns().iter().map(|other_column|{
                                    let default_self_column = format!(#default_column_format, other_column);
                                        let self_column = match other_column.as_str() {
                                            #(#match_translation)*
                                            _ => &default_self_column
                                            };
                                        format!("({}{}{} IS NOT NULL)",sql_alias, if sql_alias.is_empty() { "" } else { "." }, self_column)
                                }).collect::<Vec<String>>().join(" AND ");
                                mapper.map_field_with_options(
                                &format!("{}_", #toql_field), &none_condition,toql::sql_mapper::MapperOptions::new().preselect(true));
                           )
                        );
                    }

                    // Map joined entity

                    /*   let default_other_columns= vec![crate::util::rename("id", &toql.columns)];
                    let other_columns =  if !field.join.as_ref().unwrap().other_columns.is_empty() {
                        field.join.as_ref().unwrap().other_columns.as_ref() }
                        else {
                            &default_other_columns
                        }; */

                    // TODO Add dynamic column resolution
                    /* let join = <Language as toql::key::Key>::columns().iter().zip(&["code"]).map(|(other_column, self_column)| {
                            String::from("{alias}.language_code = language.code")
                        }); */
        /*
                     let mut join_condition: Vec<String> = self_columns
                        .iter().zip(other_columns)
                        .map(|(self_column, other_column)| {
                           /*  let auto_self_key =
                                crate::util::rename(&format!("{}_id", &field_ident), &toql.columns);
                            let this_key = this_column.as_ref().unwrap_or(&auto_self_key); */


                           // let other_key = "TODO".to_string();
                            //let other_key = &j.other_column.as_ref().unwrap_or(&default_other_column); //crate::util::rename(&j.other, &toql.columns);

                            format!(
                                "{{alias}}.{} = {}.{}",
                                self_column, join_alias, other_column
                            )
                        })
                        .collect(); */

                      // Create dynamic join condition, that takes columns for Key trait
                      // TODO integrate
                      let default_column_format = format!("{}_{{}}", field_ident);
                       let match_translation  = field.join.as_ref().unwrap().columns.iter()
                                .map(|column| {
                                    let tc = &column.this; let oc = &column.other;
                                    quote!( #oc => #tc,)
                                })
                                .collect::<Vec<_>>();
                      let join_expression_builder = quote!(
                          let join_expression = <#joined_struct_ident as toql::key::Key>::columns().iter()
                            //.zip(&[ #(#self_columns),* ])
                            .map(|other_column| {
                                let default_self_column= format!(#default_column_format, other_column);
                                let self_column= match other_column.as_str() {
                                    #(#match_translation)*
                                    _ => &default_self_column
                                };

                            format!("{}{}{} = {}.{}",sql_alias , if sql_alias.is_empty() { "" } else { "." }, self_column, #join_alias, other_column)
                            }).collect::<Vec<String>>().join(" AND ")
                        );

                        // Add additional join predicate
                        /* if let Some(predicate) = &field.join.as_ref().unwrap().on_sql {
                                join_condition.push(format!("({})", predicate.replace("..",&format!("{}.",join_alias))));
                        } */
                    let on_sql = if field.join.as_ref().unwrap().on_sql.is_some() {
                                format!(" AND ({})", &field.join.as_ref().unwrap().on_sql.as_ref().unwrap().replace("..",&format!("{}.",join_alias)))
                        } else {
                            String::from("")};

                    let format_string = format!(
                        "{}JOIN {} {} ON ({{}}{})",
                        if field.number_of_options() == 2
                            || (field.number_of_options() == 1 && field.preselect == true)
                        {
                            "LEFT "
                        } else {
                            ""
                        },
                        join_table,
                        join_alias,
                        on_sql
                    );


                    let join_clause = quote!(&format!( #format_string, join_expression));
                    let join_selected = field.number_of_options() == 0
                        || (field.number_of_options() == 1 && field.preselect == true);
                    self.field_mappings.push(quote! {
                        #join_expression_builder;
                        mapper.map_join::<#joined_struct_ident>(  #toql_field, #join_alias);
                        mapper.join( #toql_field, #join_clause, #join_selected );
                    });
                }
                // Regular field
                else if field.merge.is_none() {
                    let (base, _generic, _gegeneric) = field.get_types();

                    if base == "Vec" || base =="HashSet"  {
                        let error = format!("Missing attribute `merge`. Add `#[toql( merge()]`");
                        self.field_mappings.push(quote_spanned! {
                            field_ident.span() =>
                            compile_error!( #error);
                        });
                        return Err(());
                    }
                    if base == "VecDeque"
                        || base == "LinkedList"
                        || base == "HashMap"
                        || base == "BTreeMap"
                        || base == "BTreeSet"
                    {
                        // TODO Get types as ident to highlight type and not variable name
                        self.field_mappings.push(quote_spanned! {
                            field_ident.span() =>
                            compile_error!("Invalid collection type. Only `std::vec::Vec` and  `std::collections::HashSet` are supported.");
                        });
                        return Err(());
                    }

                    let countfilter_ident = if field.count_filter {
                        quote!( .count_filter(true))
                    } else {
                        quote!()
                    };
                    let countselect_ident = if field.count_select {
                        quote!( .count_select(true))
                    } else {
                        quote!()
                    };
                    let select_ident = if field.preselect || (base.to_string() != "Option") {
                        quote!( .preselect(true))
                    } else {
                        quote!()
                    };
                    let ignore_wc_ident = if field.ignore_wildcard {
                        quote!( .ignore_wildcard(true))
                    } else {
                        quote!()
                    };

                    let roles = &field.role;
                    let roles_ident = if roles.is_empty() {
                        quote!()
                    } else {
                        quote! { .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  }
                    };

                    let field_sql = &field.sql;
                    let sql_mapping = if field_sql.is_none() {
                        quote! {&format!("{}{}{}",sql_alias, if sql_alias.is_empty() {"" }else {"."}, #sql_field)}
                    } else {
                        quote! {&format!("({})", #field_sql .replace("..",&format!("{}.",sql_alias )))}
                    };

                    self.field_mappings.push(quote! {
                                                mapper.map_field_with_options(&format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field),
                                                #sql_mapping,toql::sql_mapper::MapperOptions::new() #select_ident #countfilter_ident #countselect_ident #ignore_wc_ident #roles_ident);
                                            }
                                );
                }*/
        Ok(())
    }

    pub(crate) fn add_merge_function(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.to_owned());
    }

    /* fn build_merge_function(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let struct_ident = self.struct_ident;
        let joined_struct_ident = field.first_non_generic_type().unwrap();
        let field_ident = &field.ident.as_ref().unwrap();
        let function_ident = syn::Ident::new(&format!("merge_{}", field_ident), Span::call_site());

    let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
    let auto_self_field= "id".to_string();
        let ref self_tuple: Vec<proc_macro2::TokenStream> = field
            .merge
            .iter()
            .map(|k| {

                let key = Ident::new(&k.this_field.as_ref().unwrap_or(&auto_self_field), Span::call_site());
                quote!(t. #key)
            })
            .collect();

        let ref other_tuple: Vec<proc_macro2::TokenStream> = field
            .merge
            .iter()
            .map(|k| {
                let key = Ident::new(&k.other_field.as_ref().unwrap_or(&auto_other_field), Span::call_site());
                quote!( o. #key )
            })
            .collect();

        let self_fnc: proc_macro2::TokenStream = if field.merge.len() == 1 {
            quote!( Option::from( #(#self_tuple)*) )
        } else {
            quote!( if #( (Option::from (#self_tuple)).or)* (None).is_some() { Option::from((#(#self_tuple),* ))} else {None} )
        };
        let other_fnc: proc_macro2::TokenStream = if field.merge.len() == 1 {
            quote!( Option::from( #(#other_tuple)*) )
        } else {
            quote!( if #( (Option::from (#other_tuple)).or)* (None).is_some() { Option::from((#(#other_tuple),* ))} else {None} )
        };

        self.merge_functions.push(quote!(
            pub fn #function_ident ( t : & mut Vec < #struct_ident > , o : Vec < #joined_struct_ident > ) {
                    toql :: merge :: merge ( t , o ,
                    | t | #self_fnc ,
                    | o | #other_fnc ,
                    | t , o |  {let t : Option<&mut Vec<#joined_struct_ident>>= Option::from( &mut t. #field_ident ); if t.is_some() { t.unwrap().push(o);}}
                    ) ;
            }
         ));
    } */

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
        //let struct_name= format!("{}", struct_ident);
        let sql_table_name = &self.rust_struct.sql_table_name;
        let sql_table_alias = &self.rust_struct.sql_table_alias;

        let merge_functions = &self.merge_functions;

        let field_mappings = &self.field_mappings;

        /*  // must be processed after first pass to make sure all keys are read
          let merge_functions = self.merge_types.map(|(merge_type, merge_field)| {
           let other_fnc =  self.key_fields.iter().zip(self.merge_args).map(|key_field, (merge_arg)|{
               let default_other_field =  format!("{}_{}", entity, key_field);
               match key_field {

                   _ => default_other_field
               }

           }
               quote!(
              pub fn #function_ident ( t : & mut Vec < #struct_ident > , o : Vec < #joined_struct_ident > ) {
                      toql :: merge :: merge ( t , o ,
                      | t | #self_fnc ,
                      | o | #other_fnc ,
                      | t , o |  {let t : Option<&mut Vec<#joined_struct_ident>>= Option::from( &mut t. #field_ident ); if t.is_some() { t.unwrap().push(o);}}
                      ) ;
              })

           });
        */

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
