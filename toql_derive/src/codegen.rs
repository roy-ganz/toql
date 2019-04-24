



use crate::annot::Toql;
use crate::annot::ToqlField;
use quote::quote;


use proc_macro2::Span;

use heck::MixedCase;
use heck::SnakeCase;
use syn::Ident;

pub(crate) struct GeneratedToql<'a> {

    struct_ident: &'a Ident,
    sql_table_name: String,
    sql_table_alias: String,
    builder_fields_struct: Ident ,                   
    builder_fields : Vec<proc_macro2::TokenStream>,
    merge_functions: Vec<proc_macro2::TokenStream>,
    field_mappings: Vec<proc_macro2::TokenStream>
}

impl<'a> GeneratedToql<'a> {

     pub(crate) fn from_toql(toql: &Toql) -> GeneratedToql {

         let tables = &toql.tables.as_ref();
        
        GeneratedToql {
            struct_ident: &toql.ident,
            sql_table_name: toql.ident.to_string(), //crate::util::rename_sql_table(&toql.ident.to_string(), tables),
            sql_table_alias: toql.ident.to_string().to_snake_case(),  //toql.alias.as_ref().unwrap_or( &toql.ident.to_string().to_snake_case()).to_string(),
            builder_fields_struct: syn::Ident::new(&format!("{}Fields", toql.ident.to_string()), Span::call_site()),
            builder_fields : Vec::new(),
            merge_functions : Vec::new(),
            field_mappings : Vec::new()
        }
    }

     pub(crate) fn add_field_mapping(&mut self, _toql: & Toql, field: &'a ToqlField) ->Result<(), ()> {

                        let field_ident = &field.ident.as_ref().unwrap();

                       
                        let toql_field =  format!("{}",field_ident).to_mixed_case();
                        
                        let tmp_string= field_ident.to_string();
                        let sql_field : &str =   match &field.column {
                            Some(string) => string,
                            None => &tmp_string
                        }; 
                        
                       if field.join.is_some() {
                           let field_name= field_ident.to_string();
                           let vk :Vec<&str>= field.join.as_ref().unwrap().split("<=").collect();
                            let joined_struct_ident = field.first_non_generic_type();
                            let joined_struct_name = field.first_non_generic_type().unwrap().to_string();
                            let join_table= &field.table.as_ref().unwrap_or(&joined_struct_name); // TODO table rename
                            let join_alias= &field.alias.as_ref().unwrap_or(&field_name); 

                            let this_key= vk[0].trim();
                            let other_key= vk[1].trim();
                            let format_string = format!("LEFT JOIN {} {} ON ({{}}.{} = {}.{})",  join_table, join_alias, this_key,join_alias, other_key );
                            let join_clause =  quote!( &format!( #format_string, sql_alias));
                            self.field_mappings.push(quote! {
                                mapper.map_join::<#joined_struct_ident>(  #toql_field, #join_alias);
                                mapper.join( #toql_field, #join_clause );
                            });
                               

                       } else if field.merge.is_none() {
                            let (base, generic, gegeneric) = field.get_types();

                            if base == "Vec" {

                                let error = format!("Missing attribute \"merge\". \
                                                    Tell Toql which field in this struct and the other struct have the same value. \
                                                    Add #[toql(merge = \"id <= {}_id\")]", _toql.ident.to_string().to_snake_case());
                                self.field_mappings.push(
                                    quote_spanned! {
                                        field_ident.span() =>
                                        compile_error!( #error);
                                    }
                                );
                                 return Err(());
                            }
                            if base == "VecDeque" 
                            || base == "LinkedList" 
                            || base == "HashMap" 
                            || base == "BTreeMap" 
                            || base == "HashSet" 
                            || base == "BTreeSet" 
                            {
                                // TODO Get types as ident and highlight type and to variable name.
                                self.field_mappings.push(
                                    quote_spanned! {
                                        field_ident.span() =>
                                        compile_error!("Invalid collection type. Only \"Vec\" is supported.");
                                    }
                                );
                                return Err(());

                             }

                            let countquery_ident = if field.count_query  { quote!( .count_query(true))} else { quote!()};
                            let select_ident = if field.select_always ||  (base.to_string() != "Option")  { quote!( .select_always(true))} else { quote!()};
                            let ignore_wc_ident = if field.ignore_wildcard {quote!( .ignore_wildcard(true))} else { quote!()};

                            let roles = &field.role;   
                            let roles_ident =  if roles.is_empty() { quote!() } else { quote!{ .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  } };  


                            let field_sql = &field.sql;             
                            let sql_mapping =  if field_sql.is_none() {
                                    quote! {&format!("{}{}{}",sql_alias, if sql_alias.is_empty() {"" }else {"."}, #sql_field)}
                            } else {
                                    quote! {& #field_sql .replace("..",&format!("{}.",sql_alias ))}
                            };

                        
                        self.field_mappings.push(quote! {
                                        mapper.map_field_with_options( 
                                        &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field), 
                                        #sql_mapping, 
                                        toql::sql_mapper::MapperOptions::new() #select_ident #countquery_ident #ignore_wc_ident #roles_ident);
                                    }
                        );
                       } 
                       Ok(())
     }

     pub(crate) fn add_merge_function(&mut self, _toql: & Toql, field: &'a ToqlField) {

         let struct_ident = self.struct_ident;
         let joined_struct_ident = field.first_non_generic_type();
         let field_ident = &field.ident.as_ref().unwrap();
         let function_ident =  syn::Ident::new( &format!("merge_{}",field_ident ), Span::call_site());

        let vk :Vec<&str>= field.merge.as_ref().expect("Merge self struct field <= other struct field").split("<=").collect();
        let self_key =syn::Ident::new(vk.get(0).unwrap().trim(), Span::call_site());
        let other_key =syn::Ident::new(vk.get(1).unwrap().trim(), Span::call_site());
        
                                       

         self.merge_functions.push(quote!(
            pub fn #function_ident ( t : & mut Vec < #struct_ident > , o : Vec < #joined_struct_ident > ) { 
                    toql :: sql_builder :: merge ( t , o , 
                    | t | Option::from( t. #self_key) , 
                    | o | Option::from( o. #other_key) , 
                    | t , o | t . #field_ident . push ( o ) 
                    ) ; 
            } 
         ));
       

     }
  
    pub(crate) fn add_field_for_builder(&mut self, _toql: & Toql, field: &'a ToqlField) {
        let field_ident= &field.ident;
        
  
    // let (field_type, generic_type, generic_geeric_type)= field_types;
     //    println!("TYPE ARE = {:?} {:?} {:?}", field_type, generic_type, generic_geeric_type);
    
       if field.join.is_none() && field.merge.is_none() {
             let toql_field = format!("{}", field_ident.as_ref().unwrap()).to_mixed_case();
        self.builder_fields.push(quote!(
            pub fn #field_ident (mut self) -> toql :: query :: Field {
                self . 0 . push_str ( #toql_field ) ;
                toql :: query :: Field :: from ( self . 0 )
            }
        ));
       } else  {
            let toql_field = format!("{}_", format!("{}", field_ident.as_ref().unwrap()).to_mixed_case());
           //let field_types =  field.get_types();
        let field_type=  field.first_non_generic_type().unwrap();

          
           let path_ident = syn::Ident::new(&field_type.to_string().to_snake_case(), Span::call_site());
           let type_ident : &Ident = field_type;
           let field_type_ident =syn::Ident::new(&format!("{}Fields", type_ident), Span::call_site());
           let path_fields_struct = quote!( super :: #path_ident :: #field_type_ident );
           
       
            self.builder_fields.push(quote!(
                        pub fn #field_ident (mut self) -> #path_fields_struct { 
                            self.0.push_str(#toql_field); 
                            #path_fields_struct ::from_path(self.0)
                        }
            ));
           
       }

    }
}
 

impl<'a> quote::ToTokens for GeneratedToql<'a> {
    fn to_tokens(&self, tokens: &mut  proc_macro2::TokenStream) {

        let struct_ident = self.struct_ident;
        let sql_table_name= &self.sql_table_name;
        let sql_table_alias= &self.sql_table_alias;

        let builder_fields_struct = &self.builder_fields_struct;
        let builder_fields= &self.builder_fields;

       
        let merge_functions= &self.merge_functions;

       
        let field_mappings= &self.field_mappings;

       


        let builder = quote!(

            impl toql::sql_mapper::Mappable for #struct_ident {
                fn insert_new_mapper(cache: &mut toql::sql_mapper::SqlMapperCache) ->  &mut toql::sql_mapper::SqlMapper {
                    let m = Self::new_mapper( #sql_table_alias);
                    cache.insert( String::from( #sql_table_name ), m);
                    cache.get_mut( #sql_table_name ).unwrap()
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

                pub fn fields ( ) -> #builder_fields_struct { #builder_fields_struct :: new ( ) }
                pub fn fields_from_path ( path : String ) -> #builder_fields_struct { #builder_fields_struct :: from_path ( path ) } 
            }


            pub struct #builder_fields_struct ( String ) ; 
            impl #builder_fields_struct { 
                pub fn new ( ) -> Self { Self :: from_path ( String :: from ( "" ) ) } 
                pub fn from_path ( path : String ) -> Self { Self ( path ) }
                #(#builder_fields)*
            }
        );




       
        println!("GEN (Builder) = {}", builder.to_string());



        tokens.extend(builder);

    }
}


 