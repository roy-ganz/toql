use crate::annot::Toql;
use crate::annot::ToqlField;
use crate::annot::KeyPair;
use quote::quote;

use proc_macro2::Span;

use heck::MixedCase;
use heck::SnakeCase;
use syn::Ident;

pub(crate) struct GeneratedToql<'a> {
    struct_ident: &'a Ident,
    vis: &'a syn::Visibility,
    sql_table_name: String,
    sql_table_alias: String,
    builder_fields_struct: Ident,
    builder_fields: Vec<proc_macro2::TokenStream>,
    merge_functions: Vec<proc_macro2::TokenStream>,
    field_mappings: Vec<proc_macro2::TokenStream>,
}

impl<'a> GeneratedToql<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedToql {
       
       let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        GeneratedToql {
            struct_ident: &toql.ident,
            vis: &toql.vis,
            sql_table_name: toql.table.clone().unwrap_or(renamed_table), //toql.ident.to_string(),
            sql_table_alias: toql
                .alias
                .clone()
                .unwrap_or(toql.ident.to_string().to_snake_case()), //  toql.ident.to_string().to_snake_case(),
            builder_fields_struct: syn::Ident::new(
                &format!("{}Fields", toql.ident.to_string()),
                Span::call_site(),
            ),
            builder_fields: Vec::new(),
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
        
        let renamed_sql_column = crate::util::rename(&field_ident.to_string(),&toql.columns);
        
        let sql_field: &str = match &field.column {
            Some(string) => string,
            None => &renamed_sql_column,
        };

        // Joined field
        if  !field.join.is_empty() {
           // let renamed_join_column = crate::util::rename_sql_column(&field_ident.to_string(),&toql.columns);
            
            let joined_struct_ident = field.first_non_generic_type();
            let joined_struct_name = field.first_non_generic_type().unwrap().to_string();
            let default_join_alias = joined_struct_name.to_snake_case();
            let renamed_join_table =
                crate::util::rename(&joined_struct_name, &toql.tables);
            let join_table = &field.table.as_ref().unwrap_or(&renamed_join_table);
            let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias); 

            let join_condition :Vec<String> =   field.join.iter().map(|j| {
                let this_key = crate::util::rename(&j.this, &toql.columns);
                let other_key = crate::util::rename(&j.other, &toql.columns);
                format!("{{alias}}.{} = {}.{}",this_key, join_alias, other_key) }).collect();

           let format_string = format!("{} JOIN {} {} ON ({})",
                if field._first_type() == "Option" {"LEFT"} else {"INNER"},
                join_table, join_alias,
                join_condition.join( " AND " )
           );

        
            let join_clause = quote!(&format!( #format_string, alias = sql_alias));
            self.field_mappings.push(quote! {
                mapper.map_join::<#joined_struct_ident>(  #toql_field, #join_alias);
                mapper.join( #toql_field, #join_clause );
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
            let select_ident = if field.select_always || (base.to_string() != "Option") {
                quote!( .select_always(true))
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

        let ref self_tuple :Vec<proc_macro2::TokenStream>= field.merge.iter().map(|k| {
                let key= Ident::new(&k.this, Span::call_site()); 
                quote!(t. #key)
        } ).collect();

         let ref other_tuple :Vec<proc_macro2::TokenStream> = field.merge.iter().map(|k| {
                let key= Ident::new(&k.other, Span::call_site()); 
                quote!( o. #key )
        } ).collect();

       let self_fnc : proc_macro2::TokenStream =  
       if field.merge.len() == 1 {
            quote!( Option::from( #(#self_tuple)*) )
       } else {
           quote!( if #( (Option::from (#self_tuple)).or)* (None).is_some() { Option::from((#(#self_tuple),* ))} else {None} )
       };
       let other_fnc : proc_macro2::TokenStream =  
       if field.merge.len() == 1 {
            quote!( Option::from( #(#other_tuple)*) )
       } else {
           quote!( if #( (Option::from (#other_tuple)).or)* (None).is_some() { Option::from((#(#other_tuple),* ))} else {None} )
       };


        self.merge_functions.push(quote!(
            pub fn #function_ident ( t : & mut Vec < #struct_ident > , o : Vec < #joined_struct_ident > ) {
                    toql :: sql_builder :: merge ( t , o ,
                    | t | #self_fnc ,
                    | o | #other_fnc ,
                    | t , o | t . #field_ident . push ( o )
                    ) ;
            }
         ));
    }

    pub(crate) fn add_field_for_builder(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let field_ident = &field.ident;
        let vis = &_toql.vis;
        if field.join.is_empty() && field.merge.is_empty() {
            let toql_field = format!("{}", field_ident.as_ref().unwrap()).to_mixed_case();
            self.builder_fields.push(quote!(
                #vis fn #field_ident (mut self) -> toql :: query :: Field {
                    self . 0 . push_str ( #toql_field ) ;
                    toql :: query :: Field :: from ( self . 0 )
                }
            ));
        } else {
            let toql_field = format!(
                "{}_",
                format!("{}", field_ident.as_ref().unwrap()).to_mixed_case()
            );

            let field_type = field.first_non_generic_type().unwrap();

           /*  let path_ident =
                syn::Ident::new(&field_type.to_string().to_snake_case(), Span::call_site()); */
            let type_ident: &Ident = field_type;
           /*  let field_type_ident =
                syn::Ident::new(&format!("{}Fields", type_ident), Span::call_site()); */
            let path_fields_struct =  quote!( < #type_ident as toql::query_builder::FieldsType>::FieldsType); //quote!( super :: #path_ident :: #field_type_ident );

            self.builder_fields.push(quote!(
                        #vis fn #field_ident (mut self) -> #path_fields_struct {
                            self.0.push_str(#toql_field);
                            #path_fields_struct ::from_path(self.0)
                        }
            ));
        }
    }
}

impl<'a> quote::ToTokens for GeneratedToql<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;
        let struct_name= format!("{}", struct_ident);
        let sql_table_name =  &self.sql_table_name;
        let sql_table_alias = &self.sql_table_alias;
        let vis = self.vis;        

        let builder_fields_struct = &self.builder_fields_struct;
        let builder_fields = &self.builder_fields;

        let merge_functions = &self.merge_functions;

        let field_mappings = &self.field_mappings;

        let builder = quote!(

            impl toql::query_builder::FieldsType for #struct_ident {
                type FieldsType = #builder_fields_struct ;
            }

            impl toql::sql_mapper::Map for #struct_ident {
                fn insert_new_mapper(cache: &mut toql::sql_mapper::SqlMapperCache) ->  &mut toql::sql_mapper::SqlMapper {
                    let m = Self::new_mapper( #sql_table_alias);
                    cache.insert( String::from( #struct_name ), m);
                    cache.get_mut( #struct_name ).unwrap()
                }

                fn new_mapper(table_alias: &str) -> toql::sql_mapper::SqlMapper {
                    let s = format!("{} {}",#sql_table_name, table_alias );
                    let mut m = toql::sql_mapper::SqlMapper::new( if table_alias.is_empty() { #sql_table_name } else { &s });
                    Self::map(&mut m, "", table_alias);
                    m
                }

                fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str, sql_alias: &str) {
                    #(#field_mappings)*
                }
            }

            impl #struct_ident {

                #(#merge_functions)*

                #vis fn fields ( ) -> #builder_fields_struct { #builder_fields_struct :: new ( ) }
                #vis fn fields_from_path ( path : String ) -> #builder_fields_struct { #builder_fields_struct :: from_path ( path ) }
            }


            #vis struct #builder_fields_struct ( String ) ;
            impl #builder_fields_struct {
                #vis fn new ( ) -> Self { Self :: from_path ( String :: from ( "" ) ) }
                #vis fn from_path ( path : String ) -> Self { Self ( path ) }
                #(#builder_fields)*
            }
        );

        println!("/* Toql (codegen) */\n {}", builder.to_string());
        
        tokens.extend(builder);
    }
}
