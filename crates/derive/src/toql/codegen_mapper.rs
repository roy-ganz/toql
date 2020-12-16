use quote::quote;

use crate::sane::{FieldKind, SqlTarget, Struct};

use proc_macro2::TokenStream;
use darling::{ Result};

pub(crate) struct CodegenMapper<'a> {
    rust_struct: &'a Struct,
    field_mappings: Vec<TokenStream>,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
    count_filter_code: TokenStream,
    key_fields: bool,
    delete_role_expr:  &'a Option<String>,
    load_role_expr:  &'a Option<String>
}

impl<'a> CodegenMapper<'a> {
    pub(crate) fn from_toql(rust_struct: &'a Struct) -> CodegenMapper {
        let mut field_mappings: Vec<TokenStream> = Vec::new();

        let rust_struct_ident = &rust_struct.rust_struct_ident;
        for selection in &rust_struct.mapped_selections {
             let name = &selection.name;
             let fields = &selection.fields;

            field_mappings.push(quote!(
                mapper.map_selection( #name, toql::fields_macro::fields!(#rust_struct_ident, #fields).list);
            ));
         }



        for mapping in &rust_struct.mapped_predicates {
            let toql_field_name = &mapping.name;
            let sql_mapping = &mapping.sql;
          
            let on_params :Vec<TokenStream>= mapping.on_param.iter().map(|p| {
                let index = &p.index;
                let name = &p.name;
                quote!(.on_param( #index, String::from(#name)))
            }).collect::<Vec<_>>();

        let countfilter_ident = if mapping.count_filter {
                    quote!( .count_filter(true))
                } else {
                    quote!()
                };

            match &mapping.handler {
                Some(handler) => {
                    field_mappings.push(quote! {
                                mapper.map_predicate_handler_with_options(
                                    #toql_field_name, 
                                    toql::sql_expr_macro::sql_expr!(#sql_mapping),
                                    #handler (), 
                                    toql::sql_mapper::predicate_options::PredicateOptions::new()  #(#on_params)* #countfilter_ident );
                            });
                }
                None => {
                    field_mappings.push(quote! {
                                mapper.map_predicate_with_options(
                                #toql_field_name, 
                                toql::sql_expr_macro::sql_expr!(#sql_mapping),
                                toql::sql_mapper::predicate_options::PredicateOptions::new() #(#on_params)* #countfilter_ident
                              );
                            });
                }
            }

           
        }
        
         let count_filter_code =  quote!();

        /*   if let Some(count_filter) = &rust_struct.count_filter {
              // Only map count filter on top entity
              quote!(
                  if toql_path.is_empty() {
                    for field in &[ #(#count_filter),*] {
                            let options = mapper.get_options(field).expect(&format!("Field {} not mapped. Skipped count filter.", &field));
                            mapper.set_options(field, options.count_filter(true));
                        }
                  }
              ) 
              quote!()
        } else {
            quote!( )
        };  */

        CodegenMapper {
            rust_struct,
            field_mappings,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
            count_filter_code,
            key_fields: true,
            delete_role_expr: &rust_struct.roles.delete,
            load_role_expr: &rust_struct.roles.load
        }
    }

    pub(crate) fn add_field_mapping(&mut self, field: &crate::sane::Field) -> Result<()> {

      

        let rust_field_name = &field.rust_field_name;
      
       let roles_ident =  match &field.roles.load {
                    Some(role) =>  quote! {  .restrict_load(toql::role_expr_macro::role_expr!(#role)) },
                    None => quote!()
                };

        // Joined field
        match &field.kind {
            FieldKind::Join(join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;
                let rust_type_ident = &field.rust_type_ident;
                let toql_field_name = &field.toql_field_name;
                let sql_join_mapper_name = &field.rust_type_name;

                let sql_join_table_name = &join_attrs.sql_join_table_name;

                if join_attrs.key {
                    if self.key_fields == false {
                          return Err(darling::Error::custom(
                                "Key must be the first fields in a struct. Move your field.".to_string(),
                            )
                        .with_span(&field.rust_field_ident));
                    } 
                } else {
                           self.key_fields = false;
                }
                

               // self.field_mappings.push(quote!( )); // use Toql field name to build join alias (prevents underscore in name)))

               // Add discriminator field for LEFT joins
                let left_join_discriminator = if field.number_of_options > 1
                    || (field.number_of_options == 1 && field.preselect == true)
                {
                    quote!(
                        .discriminator( 
                            {
                                let mut e = toql::sql_expr::SqlExpr::new();
                                <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter()
                                .for_each(| other_column |
                                {
                                    e.push_other_alias();
                                    e.push_literal(".");
                                    e.push_literal(other_column);
                                    e.push_literal(" IS NOT NULL AND ");
                                    
                                });
                                e.pop_literals(5);
                                e 
                            }
                        ) 
                    )
                } else {
                    quote!()
                }; 

                // Build predicate based on key information or custom provided column pairs
                let col_array = if join_attrs.columns.is_empty() {
                    quote!(<<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns())
                } else {
                     let other_columns: Vec<String> = join_attrs.columns
                        .iter()
                        .map(|column| String::from(column.other.as_str()))
                        .collect::<Vec<_>>();
                        quote!( [ #(String::from(#other_columns)),* ])
                };
                let on_predicate = if let Some( on ) = &join_attrs.on_sql {
                    quote!(t.extend(toql::sql_expr_macro::sql_expr!(#on)))
                } else {
                    quote!(t.pop_literals(5)) // Remove unneeded ' AND '
                };
                let join_predicate = quote!( 
                     #col_array .iter()
                    .for_each(|other_column| {
                        #default_self_column_code;
                        let self_column= #columns_map_code;
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                     #on_predicate
                     
                     );
                          

                let join_statement = if field.number_of_options == 0
                    || (field.number_of_options == 1 && field.preselect == false)
                {
                   format!("JOIN {} ", &sql_join_table_name)
                } else {
                    format!("LEFT JOIN {} ", &sql_join_table_name)
                };

                let preselect_ident = if field.preselect || (field.number_of_options == 0) {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };
                let ignore_wc_ident = if field.skip_wildcard {
                    quote!( .skip_wildcard(true))
                } else {
                    quote!()
                };
                        


                 let aux_params = join_attrs.aux_params.iter()
                    .map(|p| { let name = &p.name; let value = &p.value; quote!(.aux_param(String::from(#name), String::from(#value))) })
                    .collect::<Vec<TokenStream>>();

                // todo map handler, see regular field

                self.field_mappings.push(quote! {
                    mapper.map_join_with_options(#toql_field_name, #sql_join_mapper_name, 
                    {let mut t = toql::sql_expr::SqlExpr::literal(#join_statement); t.push_other_alias(); t }, 
                    { let mut t = toql::sql_expr::SqlExpr::new(); #join_predicate; t },
                     toql::sql_mapper::join_options::JoinOptions::new() #(#aux_params)* #preselect_ident #ignore_wc_ident #roles_ident #left_join_discriminator
                    );
                });

                if join_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Regular(regular_attrs) => {
                let toql_field_name = &field.toql_field_name;

                if regular_attrs.key {
                    if self.key_fields == false {
                          return Err(darling::Error::custom(
                                "Key must be the first fields in a struct. Move your field.".to_string(),
                            )
                        .with_span(&field.rust_field_ident));
                    } 
                } else {
                           self.key_fields = false;
                }
            
                let preselect_ident = if field.preselect || (field.number_of_options == 0) {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };
                let ignore_wc_ident = if field.skip_wildcard {
                    quote!( .skip_wildcard(true))
                } else {
                    quote!()
                };
                let query_select_ident = if field.skip_query {
                    quote!( .query_select(false))
                } else {
                    quote!()
                };


                let mut_select_ident = if regular_attrs.key {
                    quote!(.mut_select(toql_path.is_empty() || (!toql_path.is_empty() && ! toql_path.contains('_'))))
                } else {
                    quote!(.mut_select(toql_path.is_empty()))
                };

                let aux_params = regular_attrs.aux_params.iter()
                    .map(|p| { let name = &p.name; let value = &p.value; quote!(.aux_param(String::from(#name), String::from(#value))) })
                    .collect::<Vec<_>>();
             
              let sql_mapping = match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        quote! {let sql_mapping = toql::sql_expr_macro::sql_expr!( #expression);}
                    }
                    SqlTarget::Column(ref column) => {
                        quote! { let sql_mapping = #column; }
                    }
                }; 

                match &regular_attrs.handler {
                    Some(handler) => {
                         let sql_expr = match &regular_attrs.sql_target {
                            SqlTarget::Expression(ref expression) => {
                                quote! { toql::sql_expr_macro::sql_expr!( #expression)}
                            }
                            SqlTarget::Column(ref column) => {
                                quote! { toql::sql_expr::SqlExpr::aliased_column(#column) }
                            }
                        }; 
                        self.field_mappings.push(quote! {
                                #sql_mapping
                                mapper.map_handler_with_options( #toql_field_name, #sql_expr, #handler (), toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)* #preselect_ident  #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                            });
                    }
                    None => {
                          self.field_mappings.push( match &regular_attrs.sql_target {
                            SqlTarget::Expression(ref expression) => {
                                quote! {
                                    mapper.map_expr_with_options( #toql_field_name,  toql::sql_expr_macro::sql_expr!( #expression),
                                toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)*  #preselect_ident #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                                }
                            }
                            SqlTarget::Column(ref column) => {
                                quote! {
                                    mapper.map_column_with_options( #toql_field_name, #column ,
                                    toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)*  #preselect_ident #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                                 }
                            }
                        }); 
                       /*  self.field_mappings.push(quote! {
                            #sql_mapping
                                mapper.map_column_with_options( #toql_field_name, sql_mapping,
                                toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)*  #preselect_ident #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                            }); */
                    }
                };

                if regular_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Merge(ref merge_attrs) => {
                let toql_field_name = &field.toql_field_name;
                let sql_merge_mapper_name = &field.rust_type_name;

               
               
                let join_statement= if let Some(custom_join) = &merge_attrs.join_sql {
                    quote!(toql::sql_expr_macro::sql_expr!(#custom_join))
                } else {
                    //let table_name = &merge_attrs.sql_join_table_name;
                    let table_name = &self.rust_struct.sql_table_name;
                   quote!(  
                        toql::sql_expr::SqlExpr::from(vec![
                        toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                        toql::sql_expr::SqlExprToken::Literal(#table_name.to_string()),
                        toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                        toql::sql_expr::SqlExprToken::SelfAlias
                       ])
                     )
                   };

              
                // Build join predicate
                // - use custom predicate if provided
                // - build from columns, if provided
                // - build from key, if columns are missing
              
                let join_predicate=  if let Some(custom_on) = &merge_attrs.on_sql {
                     quote!( toql::sql_expr_macro::sql_expr!( #custom_on))
                } else {

                    if merge_attrs.columns.is_empty() {
                        let self_key_ident = syn::Ident::new(&format!("{}Key", &self.rust_struct.rust_struct_name), proc_macro2::Span::call_site());
                       // let type_key_ident = syn::Ident::new(&format!("{}Key", &field.rust_type_name), proc_macro2::Span::call_site());
                        quote!(  { 
                            let mut tokens: Vec<toql::sql_expr::SqlExprToken>= Vec::new();
                                <#self_key_ident as toql::key::Key>::columns().iter()
                                .zip(<#self_key_ident as toql::key::Key>::default_inverse_columns()).for_each(|(t,o)| {
                                tokens.extend(vec![toql::sql_expr::SqlExprToken::SelfAlias,
                                toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                                toql::sql_expr::SqlExprToken::Literal(t.to_string()),
                                toql::sql_expr::SqlExprToken::Literal(" = ".to_string()),
                                toql::sql_expr::SqlExprToken::OtherAlias,
                                toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                                toql::sql_expr::SqlExprToken::Literal(o.to_string()),
                                toql::sql_expr::SqlExprToken::Literal( " AND ".to_string())
                                ].into_iter())});
                                tokens.pop(); // ' AND '
                                toql::sql_expr::SqlExpr::from(tokens)
                            })
                    } else {
                     let mut default_join_predicate : Vec<TokenStream> = Vec::new();
                     default_join_predicate.push(quote!(  let mut t =  toql::sql_expr::SqlExpr::new();));
                        for m in &merge_attrs.columns {
                            let this_column = &m.this;
                            default_join_predicate.push( quote!( 
                                        t.push_self_alias(); 
                                        t.push_literal("."); 
                                        t.push_literal(#this_column); 
                                        t.push_literal(" = "); )); 
                            match &m.other {
                                crate::sane::MergeColumn::Aliased(a) => {
                                    default_join_predicate.push( quote!( t.push_literal(#a);));
                                }
                                crate::sane::MergeColumn::Unaliased(u) => {
                                    default_join_predicate.push( quote!( 
                                        t.push_other_alias(); 
                                        t.push_literal("."); 
                                        t.push_literal(#u); 
                                        t.push_literal(" AND "); ))
                                }
                            }
                        }
                        default_join_predicate.push( quote!(t.pop_literals(5); t));
                        
                        quote!(#(#default_join_predicate)*)
                    }
                    };

                self.field_mappings.push(quote! {
                        mapper.map_merge(#toql_field_name, #sql_merge_mapper_name, 
                           {#join_statement}, 
                            { #join_predicate }
                            );
                });
                    


            }
        };
        Ok(())
    }

    pub(crate) fn add_merge_function(&mut self, field: &crate::sane::Field) {
        self.merge_fields.push(field.to_owned());
    }
}

impl<'a> quote::ToTokens for CodegenMapper<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;

        let sql_table_name = &self.rust_struct.sql_table_name;
        let sql_table_alias = &self.rust_struct.sql_table_alias;

        let field_mappings = &self.field_mappings;
        let count_filter_code = &self.count_filter_code;
      
       let delete_role_code = match &self.delete_role_expr {
           Some(r) => quote!(mapper.restrict_delete( toql::role_expr_macro::role_expr!(#r)); ),
           None => quote!()
          };
       let load_role_code = match &self.delete_role_expr {
           Some(r) => quote!(mapper.restrict_load( toql::role_expr_macro::role_expr!(#r)); ),
           None => quote!()
          };

        
       
        let builder = quote!(

            impl toql::sql_mapper::mapped::Mapped for #struct_ident {

                fn type_name() -> String {
                    String::from(#struct_name)
                }

                fn table_name() -> String {
                    String::from(#sql_table_name)
                }
                fn table_alias() -> String {
                    String::from(#sql_table_alias)
                }
                fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str) -> toql::error::Result<()>{
                  /*   if toql_path.is_empty() {
                        mapper.aliased_table = mapper.translate_aliased_table(#sql_table_name, canonical_sql_alias);
                    } */
                    


                    #(#field_mappings)*

                    #count_filter_code

                    #load_role_code
                    #delete_role_code
                    Ok(())
                }
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
