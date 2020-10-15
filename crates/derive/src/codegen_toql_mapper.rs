use quote::quote;

use crate::sane::{FieldKind, SqlTarget, Struct};

use proc_macro2::TokenStream;
use darling::{Error, Result};

pub(crate) struct GeneratedToqlMapper<'a> {
    rust_struct: &'a Struct,
    field_mappings: Vec<TokenStream>,
    merge_fields: Vec<crate::sane::Field>,
    key_field_names: Vec<String>,
    count_filter_code: TokenStream
    
}

impl<'a> GeneratedToqlMapper<'a> {
    pub(crate) fn from_toql(rust_struct: &'a Struct) -> GeneratedToqlMapper {
        let mut field_mappings: Vec<TokenStream> = Vec::new();

        

        for mapping in &rust_struct.mapped_predicates {
            let toql_field_name = &mapping.name;
            let sql_mapping = &mapping.sql;
            let sql_expr = if sql_mapping.contains(".."){
                let sql_mapping = sql_mapping.replace("..", "{alias}."); 
                quote!(&format!(#sql_mapping, alias = mapper.translated_alias(&canonical_sql_alias)))
            } else {
                quote!(#sql_mapping)
            };

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
                                    &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                    #sql_expr ,
                                    #handler (), 
                                    toql::sql_mapper::PredicateOptions::new()  #(#on_params)* #countfilter_ident );
                            });
                }
                None => {
                    field_mappings.push(quote! {
                                mapper.map_predicate_with_options(
                                    &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                                #sql_expr,
                                toql::sql_mapper::PredicateOptions::new() #(#on_params)* #countfilter_ident
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

        GeneratedToqlMapper {
            rust_struct,
            field_mappings,
            merge_fields: Vec::new(),
            key_field_names: Vec::new(),
            count_filter_code
        }
    }

    pub(crate) fn add_field_mapping(&mut self, field: &crate::sane::Field) -> Result<()> {

      

        let rust_field_name = &field.rust_field_name;
      

        // Joined field
        match &field.kind {
            FieldKind::Join(join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;
                let rust_type_ident = &field.rust_type_ident;
                let toql_field_name = &field.toql_field_name;
                let sql_join_mapper_name = &field.rust_type_name;

                let sql_join_table_name = &join_attrs.sql_join_table_name;
                

               // self.field_mappings.push(quote!( )); // use Toql field name to build join alias (prevents underscore in name)))

               // Add discriminator field for LEFT joins
                let left_join_discriminator = if field.number_of_options > 1
                    || (field.number_of_options == 1 && field.preselect == true)
                {
                    quote!(
                        .discriminator( toql::sql_expr_parser::SqlExprParser::parse( {
                            <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().map(|other_column|{
                                    format!("(...{} IS NOT NULL)", &other_column)
                            }).collect::<Vec<String>>().join(" AND ").as_str()  

                        })? 
                        ) 
                    )
                } else {
                    quote!()
                }; 

                // If no columns are provided, use default
             /*    let col_array = if join_attrs.columns.is_empty() {
                    quote!(<<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns())
                } else {
                     let other_columns: Vec<String> = join_attrs.columns
                        .iter()
                        .map(|column| String::from(column.other.as_str()))
                        .collect::<Vec<_>>();
                        quote!( [ #(String::from(#other_columns)),* ])
                }; */
/* 
                let join_expression_builder = quote!(
                    
                //  let join_alias = format!("{}_{}",canonical_sql_alias, #join_alias);

                 // #left_join_discriminator
                  //let join_expression = <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter()
                 /*  let join_expression =  #col_array .iter()
                    .map(|other_column| {
                        #default_self_column_code;
                        let self_column= #columns_map_code;
                        format!("{} = {}", & mapper.translate_aliased_column(canonical_sql_alias, &self_column),
                        & mapper.translate_aliased_column(&join_alias,other_column))
                    }).collect::<Vec<String>>().join(" AND ")
                ); */
                  let join_expression =  #col_array .iter()
                    .map(|other_column| {
                        #default_self_column_code;
                        let self_column= #columns_map_code;
                        format!("{} = {}",  &self_column,&other_column)
                    }).collect::<Vec<String>>().join(" AND ")
                ); */
    // Alias resolving
             /*    let on_sql = if let Some(ref sql) = &join_attrs.on_sql {
                    format!(" AND ({})", sql.replace("...", "{join_alias}.").replace("..", "{alias}."))
                } else {
                    String::from("")
                }; */

              

                
                // Avoid unused arguments by consuming them with zero length
                // let join_predicate_format = format!("{{join}}{}{{join_alias:.0}}{{alias:.0}}", on_sql);

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
                    quote!(t.extend(toql::sql_expr_parser::SqlExprParser::parse(#on)?))
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
                
              /*   let mut_select_ident = if join_attrs.key {
                    quote!(.mut_select(toql_path.is_empty() || (!toql_path.is_empty() && ! toql_path.contains('_'))))
                } else {
                    quote!(.mut_select(toql_path.is_empty()))
                };
 */

                let roles = &field.load_roles;
                let roles_ident = if roles.is_empty() {
                    quote!()
                } else {
                    quote! { .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  }
                };

                 let aux_params = join_attrs.aux_params.iter()
                    .map(|p| { let name = &p.name; let value = &p.value; quote!(.aux_param(String::from(#name), String::from(#value))) })
                    .collect::<Vec<TokenStream>>();

                // todo map handler, see regular field

                self.field_mappings.push(quote! {

                    /* mapper.map_join_with_options(#toql_field_name, #sql_join_mapper_name, 
                    toql::sql_expr_parser::SqlExprParser::parse(#join_statement)?, 
                    toql::sql_expr_parser::SqlExprParser::parse( #join_predicate)?,
                     toql::sql_mapper::join_options::JoinOptions::new() #(#aux_params)* #preselect_ident #ignore_wc_ident #roles_ident #left_join_discriminator
                    ); */
                    mapper.map_join_with_options(#toql_field_name, #sql_join_mapper_name, 
                    {let mut t = toql::sql_expr::SqlExpr::literal(#join_statement); t.push_other_alias(); t }, 
                    { let mut t = toql::sql_expr::SqlExpr::new(); #join_predicate; t },
                     toql::sql_mapper::join_options::JoinOptions::new() #(#aux_params)* #preselect_ident #ignore_wc_ident #roles_ident #left_join_discriminator
                    );

                  /*   #join_expression_builder;
                    mapper.map_join::<#rust_type_ident>( &format!("{}{}{}",
                        toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                        &join_alias);

                    let aliased_table = mapper.translate_aliased_table(#sql_join_table_name, &join_alias);
                    mapper.join_with_options( &format!("{}{}{}",toql_path,if toql_path.is_empty() {"" }else {"_"}, #toql_field_name), 
                     #join_type,
                     &aliased_table,
                     #join_predicate,
                     toql::sql_mapper::JoinOptions::new() #(#aux_params)* #preselect_ident #ignore_wc_ident #roles_ident); */
                });

                if join_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Regular(regular_attrs) => {
                let toql_field_name = &field.toql_field_name;
              /*   let countfilter_ident = if regular_attrs.count_filter {
                    quote!( .count_filter(true))
                } else {
                    quote!()
                };
                let countselect_ident = if regular_attrs.count_select {
                    quote!( .count_select(true))
                } else {
                    quote!()
                }; */
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

                let roles = &field.load_roles;
                let roles_ident = if roles.is_empty() {
                    quote!()
                } else {
                    quote! { .restrict_roles( [ #(String::from(#roles)),* ].iter().cloned().collect())  }
                };

                let aux_params = regular_attrs.aux_params.iter()
                    .map(|p| { let name = &p.name; let value = &p.value; quote!(.aux_param(String::from(#name), String::from(#value))) })
                    .collect::<Vec<_>>();
             
              let sql_mapping = match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        quote! {let sql_mapping = toql::sql_expr_parser::SqlExprParser::parse( #expression)?;}
                    }
                    SqlTarget::Column(ref column) => {
                        quote! { let sql_mapping = #column; }
                    }
                }; 

                match &regular_attrs.handler {
                    Some(handler) => {
                        self.field_mappings.push(quote! {
                                #sql_mapping
                                mapper.map_handler_with_options( #toql_field_name, sql_mapping, #handler (), toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)* #preselect_ident  #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                            });
                    }
                    None => {
                        self.field_mappings.push(quote! {
                            #sql_mapping
                                mapper.map_column_with_options( #toql_field_name, sql_mapping,
                                toql::sql_mapper::field_options::FieldOptions::new() #(#aux_params)*  #preselect_ident #ignore_wc_ident #roles_ident #mut_select_ident #query_select_ident);
                            });
                    }
                };

                if regular_attrs.key {
                    self.key_field_names.push(rust_field_name.to_string());
                }
            }
            FieldKind::Merge(ref merge_attrs) => {
                let toql_field_name = &field.toql_field_name;
                let sql_merge_mapper_name = &field.rust_type_name;

                // simple join statement or custom join statement
                let join_statement= &merge_attrs.join_sql; 

                // From keys or custom predicate
                
                let join_predicate= &merge_attrs.on_sql; // from on predicate or 

                self.field_mappings.push(quote! {
                        mapper.map_merge(#toql_field_name, #sql_merge_mapper_name, 
                            toql::sql_expr_parser::SqlExprParser::parse(#join_statement)?, 
                            toql::sql_expr_parser::SqlExprParser::parse( #join_predicate)?
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

impl<'a> quote::ToTokens for GeneratedToqlMapper<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;

        let sql_table_name = &self.rust_struct.sql_table_name;
        let sql_table_alias = &self.rust_struct.sql_table_alias;

        let field_mappings = &self.field_mappings;
        let count_filter_code = &self.count_filter_code;
       
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
                fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str, canonical_sql_alias: &str) -> toql::error::Result<()>{
                  /*   if toql_path.is_empty() {
                        mapper.aliased_table = mapper.translate_aliased_table(#sql_table_name, canonical_sql_alias);
                    } */
                    

                    #(#field_mappings)*

                    #count_filter_code
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
