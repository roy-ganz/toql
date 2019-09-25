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
        if !field.sql_join.is_empty() {
            // let renamed_join_column = crate::util::rename_sql_column(&field_ident.to_string(),&toql.columns);

            let joined_struct_ident = field.first_non_generic_type();
            let joined_struct_name = field.first_non_generic_type().unwrap().to_string();
            let default_join_alias = joined_struct_name.to_snake_case();
            let renamed_join_table = crate::util::rename(&joined_struct_name, &toql.tables);
            let join_table = &field.table.as_ref().unwrap_or(&renamed_join_table);
            let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

            // Map field with none condition
            // if this condition is true, the following joined entity is NONE

            if field.number_of_options() > 0 {
                let none_condition: String = field
                    .sql_join
                    .iter()
                    .map(|j| {
                        let auto_self_key =
                            crate::util::rename(&field_ident.to_string(), &toql.columns);
                        let this_key = j.this.as_ref().unwrap_or(&auto_self_key);
                        format!("({{alias}}{{sep}}{} IS NOT NULL)", this_key)
                    })
                    .collect::<Vec<String>>()
                    .join(" AND ");

                self.field_mappings.push(quote!(
                mapper.map_field_with_options(
                &format!("{}_", #toql_field),
                    &format!(#none_condition,
                        sep = if sql_alias.is_empty() { "" } else { "." },
                        alias = sql_alias),
                        toql::sql_mapper::MapperOptions::new().preselect(true)
                    );

                    ));
            }

            // Map join field
            let join_condition: Vec<String> = field
                .sql_join
                .iter()
                .map(|j| {
                    let auto_self_key =
                        crate::util::rename(&field_ident.to_string(), &toql.columns);
                    let this_key = j.this.as_ref().unwrap_or(&auto_self_key);
                    let other_key = &j.other; //crate::util::rename(&j.other, &toql.columns);
                    let on = if let Some(predicate) = &j.on {
                        format!(" AND ({})", predicate)
                    } else {
                        String::from("")
                    };
                    format!(
                        "{{alias}}.{} = {}.{}{}",
                        this_key, join_alias, other_key, on
                    )
                })
                .collect();

            let format_string = format!(
                "{}JOIN {} {} ON ({})",
                if field.number_of_options() == 2
                    || (field.number_of_options() == 1 && field.preselect == true)
                {
                    "LEFT "
                } else {
                    ""
                },
                join_table,
                join_alias,
                join_condition.join(" AND ")
            );

            let join_clause = quote!(&format!( #format_string, alias = sql_alias));
            let join_selected = field.number_of_options() == 0
                || (field.number_of_options() == 1 && field.preselect == true);
            self.field_mappings.push(quote! {
                mapper.map_join::<#joined_struct_ident>(  #toql_field, #join_alias);
                mapper.join( #toql_field, #join_clause, #join_selected );
            });
        }
        // Regular field
        else if field.merge.is_empty() {
            let (base, _generic, _gegeneric) = field.get_types();

            if base == "Vec" {
                let error = format!("Missing attribute `merge`. \
                                     Tell Toql which field in this struct and the other struct share the same value. \
                                     Add `#[toql( merge(self=\"id\", other=\" {}_id\") )]`", toql.ident.to_string().to_snake_case());
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
                || base == "HashSet"
                || base == "BTreeSet"
            {
                // TODO Get types as ident to highlight type and not variable name
                self.field_mappings.push(quote_spanned! {
                    field_ident.span() =>
                    compile_error!("Invalid collection type. Only `std::vec::Vec` is supported.");
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
                quote! {& #field_sql .replace("..",&format!("{}.",sql_alias ))}
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
        let joined_struct_ident = field.first_non_generic_type();
        let field_ident = &field.ident.as_ref().unwrap();
        let function_ident = syn::Ident::new(&format!("merge_{}", field_ident), Span::call_site());

        let ref self_tuple: Vec<proc_macro2::TokenStream> = field
            .merge
            .iter()
            .map(|k| {
                let key = Ident::new(&k.this, Span::call_site());
                quote!(t. #key)
            })
            .collect();

        let ref other_tuple: Vec<proc_macro2::TokenStream> = field
            .merge
            .iter()
            .map(|k| {
                let key = Ident::new(&k.other, Span::call_site());
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
