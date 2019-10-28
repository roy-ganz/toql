use crate::annot::Toql;
use crate::annot::ToqlField;
use quote::quote;

use proc_macro2::Span;

use heck::MixedCase;
use heck::SnakeCase;
use syn::Ident;

pub(crate) struct GeneratedToqlMapper<'a> {
    struct_ident: &'a Ident,

    sql_table_name: String,
    sql_table_alias: String,

    merge_functions: Vec<proc_macro2::TokenStream>,
    field_mappings: Vec<proc_macro2::TokenStream>,
}

impl<'a> GeneratedToqlMapper<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedToqlMapper {
        let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        GeneratedToqlMapper {
            struct_ident: &toql.ident,

            sql_table_name: toql.table.clone().unwrap_or(renamed_table), //toql.ident.to_string(),
            sql_table_alias: toql
                .alias
                .clone()
                .unwrap_or(toql.ident.to_string().to_snake_case()), //  toql.ident.to_string().to_snake_case(),
            merge_functions: Vec::new(),
            field_mappings: Vec::new(),
        }
    }

    pub(crate) fn add_field_mapping(
        &mut self,
        toql: &Toql,
        field: &'a ToqlField,
    ) -> Result<(), ()> {
        let field_ident = &field.ident.as_ref().unwrap();

        let toql_field = format!("{}", field_ident).to_mixed_case();

        let renamed_sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

        let sql_field: &str = match &field.column {
            Some(string) => string,
            None => &renamed_sql_column,
        };

        // Joined field
        if field.join.is_some() {
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
        else if field.merge.is_empty() {
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
        }
        Ok(())
    }

    pub(crate) fn add_merge_function(&mut self, _toql: &Toql, field: &'a ToqlField) {
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
    }
}

impl<'a> quote::ToTokens for GeneratedToqlMapper<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;
        //let struct_name= format!("{}", struct_ident);
        let sql_table_name = &self.sql_table_name;
        let sql_table_alias = &self.sql_table_alias;

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
            &self.struct_ident,
            builder.to_string()
        );

        tokens.extend(builder);
    }
}
