/*
* Generation functions for toql derive
*
*/


use crate::annot::Toql;
use crate::annot::ToqlField;
use syn::Ident;
use proc_macro2::Span;
use heck::MixedCase;



pub(crate) struct GeneratedMysql<'a> {

    struct_ident: &'a Ident,
    mysql_deserialize_fields: Vec<proc_macro2::TokenStream>,
    path_loaders: Vec<proc_macro2::TokenStream>,
    ignored_paths: Vec<proc_macro2::TokenStream>,
    merge_one_predicates: Vec<proc_macro2::TokenStream>,
    merge_many_predicates: Vec<proc_macro2::TokenStream>,
    
}

impl<'a> GeneratedMysql<'a> {
     pub(crate) fn from_toql(toql: &Toql) -> GeneratedMysql {
        
        GeneratedMysql {
            struct_ident: &toql.ident,
            mysql_deserialize_fields: Vec::new(),
            path_loaders: Vec::new(),
            ignored_paths: Vec::new(),
            merge_one_predicates: Vec::new(),
            merge_many_predicates: Vec::new()
        }
     }

    pub(crate) fn add_mysql_deserialize(&mut self, _toql: & Toql, field: &'a ToqlField) {
         let field_ident= &field.ident;

         // Regular fields
         if field.join.is_none() && field.merge.is_none()   {
            
            let assignment = if self.mysql_deserialize_fields.is_empty()  {quote!(*i) }else { quote!({*i +=1; * i })};
            self.mysql_deserialize_fields.push( quote!(
                    #field_ident : row . take ( #assignment ) . unwrap ( ) 
            ));

        } 
        // Joined fields
        else if field.join.is_some() {
            
            
            let join_type= field.first_non_generic_type();
            let assignment = if self.mysql_deserialize_fields.is_empty()  {quote!(i) }else { quote!({*i +=1; i })};
            
            // If join is optional, assign None if key column is NULL, otherwise assign deserialize nomally
            if field._first_type() == "Option" {
               let vk: Vec<&str> = field.join.as_ref().unwrap().split("<=").collect();
                let other_key = crate::util::rename_sql_column(vk[1].trim(),&_toql.columns).unwrap();// TODO RESULT
                self.mysql_deserialize_fields.push( quote!(
                #field_ident : if toql::mysql::is_null(&row, #other_key) {None} else {Some (< #join_type > :: from_row_with_index ( & mut row , #assignment ) ? ) }
                ));
            } else {
            self.mysql_deserialize_fields.push( quote!(
                #field_ident :  < #join_type > :: from_row_with_index ( & mut row , #assignment ) ? 
            ));
            }
        } 
        // Merged fields
        else {
            self.mysql_deserialize_fields.push( quote!(
                #field_ident : Vec::new()
            ));

        }
    }
    pub(crate) fn add_merge_predicates(&mut self, _toql: & Toql, field: &'a ToqlField) {
         
        let field_name= &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();
        let vk :Vec<&str>= field.merge.as_ref().expect("Merge self struct field <= other struct field").split("<=").collect();
        let toql_merge_field =format!("{}_{}",toql_field, vk.get(1).unwrap().trim().to_mixed_case()); 
        let merge_struct_key_ident = Ident::new( vk.get(0).unwrap().trim(), Span::call_site());

        self.merge_one_predicates.push( quote!(
                    query.and(toql::query::Field::from(#toql_merge_field).eq( entity. #merge_struct_key_ident));
        ));

        self.merge_many_predicates.push( quote!(
                   query.and(toql::query::Field::from(#toql_merge_field).ins(entities.iter().map(|entity| entity. #merge_struct_key_ident).collect()));
        ));

    }
    pub(crate) fn add_ignored_path(&mut self, _toql: & Toql, field: &'a ToqlField) {
        let field_name= &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();

         self.ignored_paths.push( quote!(
                    .ignore_path( #toql_field)));

    }
     pub(crate) fn add_path_loader(&mut self, _toql: & Toql, field: &'a ToqlField) {
        let struct_ident= &self.struct_ident;
        let field_ident= &field.ident;
        let field_name= &field.ident.as_ref().unwrap().to_string();
        let toql_field = field_name.to_mixed_case();
        let merge_type = field.first_non_generic_type().unwrap();

        let merge_function = Ident::new(&format!("merge_{}", &field.ident.as_ref().unwrap()), Span::call_site());

         self.path_loaders.push( quote!(
                let #field_ident = #merge_type ::load_path_from_mysql(#toql_field, &query, mappers, conn);
                #struct_ident :: #merge_function (&mut entities, #field_ident);
         ));
     }
    pub fn loader_functions(&self ) -> proc_macro2::TokenStream {
            let struct_ident= &self.struct_ident;
            let struct_name = &self.struct_ident.to_string();
            let path_loaders= &self.path_loaders;
            let ignored_paths= &self.ignored_paths;
            let merge_one_predicates= &self.merge_one_predicates;
            let merge_many_predicates= &self.merge_many_predicates;

            let load_dependencies_from_mysql = if path_loaders.is_empty() {quote!(
                pub fn load_dependencies_from_mysql(mut _entities: &mut Vec< #struct_ident >, 
                _query: &mut toql::query::Query,  _mappers: &toql::sql_mapper::SqlMapperCache, _conn: &mut mysql::Conn) {}
            )} else {quote!(
                pub fn load_dependencies_from_mysql(mut entities: &mut Vec< #struct_ident >, query: &mut toql::query::Query,  mappers: &toql::sql_mapper::SqlMapperCache, conn: &mut mysql::Conn) 
                {
                    #(#path_loaders)*
                }
            )};

        quote!( 
            impl #struct_ident {
               
                pub fn load_path_from_mysql(path: &str, query: &toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache,  conn: &mut mysql::Conn) 
                -> std::vec::Vec< #struct_ident > 
                {
                    let mapper = mappers.get( #struct_name ).unwrap();
                    let result = toql::sql_builder::SqlBuilder::new().build_path(path, mapper, &query).unwrap(); 
                    toql::log::info!("SQL = \"{}\" with params {:?}", result.to_sql(), result.params());
                    if result.is_empty() {
                        vec![]
                    } else {
                        let entities_stmt = conn.prep_exec(result.to_sql(), result.params());
                        let entities = toql::mysql::row::load::< #struct_ident >(entities_stmt).unwrap();
                        entities
                    }
                }
            
           
                #load_dependencies_from_mysql
            }
            impl toql::mysql::load::Load<#struct_ident> for #struct_ident
            {
                fn load_one(mut query: &mut toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache, conn: &mut mysql::Conn, distinct:bool ) 
                    -> Result<# struct_ident , toql::load::LoadError> 
                {
                    let mapper= mappers.get( #struct_name).unwrap();
                
                    let hint = String::from(if distinct { "DISTINCT" } else {""});
                
                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query).unwrap();
                    
                    toql::log::info!("SQL = \"{}\" with params {:?}", result.to_sql_for_mysql(&hint, 0, 2), result.params());
                    
                    

                    let entities_stmt = conn.prep_exec(result.to_sql_for_mysql( &hint, 0, 2), result.params());
                    let mut entities = toql::mysql::row::load::< #struct_ident >(entities_stmt).unwrap();

                    if entities.len() > 1 {
                        return Err(toql::load::LoadError::NotUnique);
                    } else if entities.is_empty() {
                        return Err(toql::load::LoadError::NotFound);
                    }
                                
                    // Restrict dependencies to parent entity
                    // query.and( "parent_child_id eq XX" )
                    let entity = entities.get(0).unwrap();
                    #(#merge_one_predicates)*
                    #struct_ident ::load_dependencies_from_mysql(&mut entities, &mut query, mappers, conn);

                    Ok(entities.pop().unwrap())
                }
                
                
                fn load_many(mut query: &mut toql::query::Query, mappers: &toql::sql_mapper::SqlMapperCache, 
                mut conn: &mut mysql::Conn, distinct:bool, count:bool, first:u64, max:u16) 
                -> Result<(std::vec::Vec< #struct_ident >, Option<(u32, u32)>), mysql::error::Error> {

                    let mapper = mappers.get( #struct_name) .unwrap();
                    // load base entities
                
                    let mut hint = String::from( if count {"SQL_CALC_FOUND_ROWS" }else{""}); 
                    
                    if distinct {
                        if !hint.is_empty() {
                            hint.push(' ');
                        }
                        hint.push_str("DISTINCT");
                    }
                    
                    let result = toql::sql_builder::SqlBuilder::new()
                    #(#ignored_paths)*
                    .build(mapper, &query).unwrap();

                    toql::log::info!("SQL = \"{}\" with params {:?}", result.to_sql_for_mysql(&hint, first, max), result.params());
                    let entities_stmt = conn.prep_exec(result.to_sql_for_mysql( &hint, first, max), result.params());
                    let mut entities = toql::mysql::row::load::< #struct_ident >(entities_stmt).unwrap();
                    let mut count_result = None;
                    
                    // Get count values
                    if count {
                        toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
                        let r = conn.query("SELECT FOUND_ROWS();").unwrap();
                        let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();

                        let result = toql::sql_builder::SqlBuilder::new().build_count(mapper, &query).unwrap();
                        toql::log::info!("SQL = \"{}\" with params {:?}", result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params());
                        conn.prep_exec(result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0), result.params()).expect("SQL error"); // don't select any rows
                        toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
                        let r = conn.query("SELECT FOUND_ROWS();").unwrap();
                        let filtered_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
                        count_result = Some((total_count ,filtered_count))
                    }

                    // Resolve dependencies
                    // Restrict query to keys     
                    #(#merge_many_predicates)*
                
                    #struct_ident ::load_dependencies_from_mysql(&mut entities, &mut query, mappers, &mut conn);
                    
                    Ok((entities, count_result))
                }
            }
            
        )
            
    }
}


impl<'a> quote::ToTokens for GeneratedMysql<'a> {
    fn to_tokens(&self, tokens: &mut  proc_macro2::TokenStream) {

    let struct_ident = self.struct_ident;
    let loader = self.loader_functions() ;
    let mysql_deserialize_fields= &self.mysql_deserialize_fields;

        let mysql= quote!(
           
            #loader
            

            impl toql :: mysql :: row:: FromResultRow < #struct_ident > for #struct_ident { 
            fn from_row_with_index ( mut row : & mut mysql :: Row , i : & mut usize ) -> Result < #struct_ident , mysql :: error :: Error > {
                Ok ( #struct_ident { 
                    #(#mysql_deserialize_fields),*

                })
            }
            }

        );

       
        println!("/* Toql (codegen_mysql) */\n {}", mysql.to_string());
        

        tokens.extend(mysql);

    }

}