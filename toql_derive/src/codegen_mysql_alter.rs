/*
* Generation functions for toql derive
*
*/


use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use std::collections::HashMap;
use syn::Ident;
use syn::Type;

pub(crate) struct GeneratedMysqlAlter<'a> {
    struct_ident: &'a Ident,
    sql_table_name: Ident,

    alter_keys: HashMap<&'a Ident, &'a Type>,
    alter_insert_params: Vec<proc_macro2::TokenStream>,
    alter_update_params: Vec<proc_macro2::TokenStream>,
    alter_delete_params: Vec<proc_macro2::TokenStream>,
    alter_delete_many_params: Vec<proc_macro2::TokenStream>,
    alter_columns: Vec<String>,
    alter_update_fnc: Vec<proc_macro2::TokenStream>,
}

impl<'a> GeneratedMysqlAlter<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedMysqlAlter {
        let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);

        GeneratedMysqlAlter {
            struct_ident: &toql.ident,
            sql_table_name: Ident::new(
                &toql.table.clone().unwrap_or(renamed_table),
                Span::call_site(),
            ),
            alter_keys: HashMap::new(),
            alter_insert_params: Vec::new(),
            alter_update_params: Vec::new(),
            alter_delete_params: Vec::new(),
            alter_delete_many_params: Vec::new(),
            alter_columns: Vec::new(),
            alter_update_fnc: Vec::new(),
        }
    }

    pub(crate) fn add_alter_field(&mut self, toql: &Toql, field: &'a ToqlField) {
        
        let field_ident = field.ident.as_ref().unwrap();
        

        // Field used as key
        if field.alter_key {
            let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
            let del_many_fmt = format!("{}{{}}", sql_column);
            self.alter_delete_params
                .push(quote!( #sql_column => entity. #field_ident. to_owned()));
            self.alter_delete_many_params.push( quote!( params.push((format!( #del_many_fmt, i), entity. #field_ident.to_string().to_owned()))));
            self.alter_update_params
                .push(quote!( #sql_column => entity . #field_ident .to_owned()));
            self.alter_keys
                .insert(field.ident.as_ref().unwrap(), &field.ty);
        }
        // Regular field
        else if field.merge.is_empty() && field.join.is_empty() && field.sql.is_none() {
            let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);

            let set_statement = format!("SET {} = :{}", &sql_column, &sql_column);

           
                       
            // Option Value
            if field._first_type() == "Option"  && !field.select_always {

                 let unwrapping = if 2 == field.number_of_options(){
                                            quote!(.map_or(String::from("null"), |x| x.to_string()))
                                        } else { quote!() };
                        
                self.alter_update_fnc.push(quote!(
                    if entity. #field_ident .is_some() {
                        update_stmt.push_str( #set_statement);
                    }
                ));
                self.alter_insert_params
                    .push(quote!( params.push( entity . #field_ident .as_ref().unwrap() #unwrapping .to_string())));
                // Update params must be by ref   
                self.alter_update_params
                    .push(quote!( #sql_column => entity . #field_ident .as_ref().unwrap().to_owned()));
            } else {
                // For Insert we need to unwrap null fields
                let unwrapping = if field._first_type() == "Option" {
                      quote!(.map_or(String::from("null"), |x| x.to_string()))    
                } else { quote!()   };

                self.alter_update_fnc.push(quote!(
                        update_stmt.push_str( #set_statement);
                ));
                self.alter_insert_params
                    .push(quote!(params.push(entity . #field_ident #unwrapping .to_string())));
                self.alter_update_params
                    .push(quote!( #sql_column => entity . #field_ident .to_owned()));
            }

            self.alter_columns.push(sql_column);
        }
        // Join fields
        else if !field.join.is_empty() {
            for j in &field.join {
                //let sql_column= crate::util::rename(&field_ident.to_string(),&toql.columns);

                let self_field = &j.this;
                let self_column = crate::util::rename(&self_field, &toql.columns);
                let other_field = Ident::new(&j.other, Span::call_site());
                let set_statement = format!("SET {} = :{}", &self_column, &self_column);

                
                if field._first_type() == "Option" {
                    self.alter_update_fnc.push(quote!(
                        if entity. #field_ident .is_some() {
                            update_stmt.push_str( #set_statement);
                        }
                    ));
                    self.alter_insert_params.push( quote!( params.push(entity. #field_ident .as_ref().map_or( String::from("null"), |e| e. #other_field .to_string()))));
                    self.alter_update_params.push( quote!( #self_column => entity. #field_ident .as_ref().map_or(None, |e| Some(e. #other_field .to_owned()))));
                } else {
                    self.alter_update_fnc.push(quote!(
                        update_stmt.push_str( #set_statement);
                    ));
                    // Non-string fields will not turn into owned string, so add to_owned() for string fields
                    self.alter_insert_params
                        .push(quote!( params.push(entity. #field_ident . #other_field .to_string().to_owned()))); 
                    self.alter_update_params
                        .push(quote!( #self_column => entity. #field_ident . #other_field .to_owned()));
                }
                self.alter_columns.push(self_column);
            }
        }

    }
}
impl<'a> quote::ToTokens for GeneratedMysqlAlter<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;


     let mut alter_delete_key_comparison: Vec<proc_macro2::TokenStream> = Vec::new();
        for key in self.alter_keys.keys() {
            let fmt = format!("{}{{i}} = :{}{{i}}", key, key);
            alter_delete_key_comparison
                .push(quote!( delete_stmt.push_str( &format!(#fmt, i = i) ); ));
            alter_delete_key_comparison.push(quote!( delete_stmt.push_str(" AND "); ));
        }
        alter_delete_key_comparison.pop(); // remove last and

        let alter_key_comparison = self
            .alter_keys
            .keys()
            .map(|k| format!("{} = :{}", k, k))
            .collect::<Vec<String>>()
            .join(" AND ");

        
        let alter_update_fnc = &self.alter_update_fnc;
        let alter_update_params = &self.alter_update_params;
        let alter_delete_params = &self.alter_delete_params;
        let alter_delete_many_params = &self.alter_delete_many_params;
        let alter_insert_params = &self.alter_insert_params;

        let alter = if self.alter_keys.is_empty() {
            quote_spanned! {
                struct_ident.span() =>
                compile_error!( "cannot find key(s) to insert, update and delete: add `#[toql(alter_key)]` to at least one field in struct");
            }
        } else {
            let update_statement = format!("UPDATE {}", self.sql_table_name);

            let insert_cols = self
                .alter_columns
                .iter()
                .map(|_v| "?".to_string())
                .collect::<Vec<String>>()
                .join(",");
            let insert_statement = format!(
                "INSERT INTO {} ({}) VALUES",
                self.sql_table_name,
                self.alter_columns.join(",")
            );

            let update_where_statement = format!(" WHERE {}", alter_key_comparison);
            let delete_statement = format!(
                "DELETE FROM {} WHERE {}",
                self.sql_table_name, alter_key_comparison
            );
            let delete_many_statement = format!("DELETE FROM {}", self.sql_table_name);

            quote! {
                impl<'a> toql::mysql::alter::Alter<'a, #struct_ident> for #struct_ident {
                 

                     fn insert_one(entity: & #struct_ident, conn: &mut mysql::Conn) -> Result<u64, toql::error::ToqlError> {
                        Self::insert_many(std::iter::once(entity), conn)
                    }

                     fn insert_many<I>(entities: I, conn: &mut mysql::Conn)
                     -> Result<u64, toql::error::ToqlError>
                     where I: Iterator<Item=&'a #struct_ident>
                     {
                    use mysql::params;

                            let mut params= Vec::new();
                            let mut insert_stmt = String::from( #insert_statement);

                            for entity in entities {
                                insert_stmt.push('(');
                                insert_stmt.push_str( #insert_cols );
                                insert_stmt.push(')');
                                #(#alter_insert_params ;)*
                               
                            }
                            if params.is_empty() {return Ok(0);}
                            toql::log::info!("Sql `{}` with params {:?}", insert_stmt, params);
                            let mut stmt = conn.prepare(insert_stmt)?;
                            let res= stmt.execute(params)?;
                            Ok(res.last_insert_id())

                    }

                    fn update_one(  entity: & #struct_ident, conn: &mut mysql::Conn)  -> Result<u64, toql::error::ToqlError>{
                        use mysql::params;
                        let mut update_stmt = String::from( #update_statement);

                        #(#alter_update_fnc)*

                        update_stmt.push_str(#update_where_statement);

                        let params= mysql::params!{  #(#alter_update_params),* };

                        toql::log::info!("Sql `{}` with params {:?}", update_stmt, params);

                        // set only
                        let mut stmt = conn.prepare(&update_stmt)?;

                        //params
                        let res = stmt.execute(params)?;

                        Ok(res.affected_rows())
                    }
                    fn update_many<I>(entities:I, conn: &mut mysql::Conn)
                    -> Result<u64, toql::error::ToqlError>
                    where I: Iterator<Item=&'a #struct_ident>
                    {

                        let mut x = 0;

                        for entity in entities{
                            x += Self::update_one(entity, conn)?
                        }
                        Ok(x)
                    }
                    fn delete_one(  entity: & #struct_ident, conn: &mut mysql::Conn ) -> Result<u64, toql::error::ToqlError>{
                        use mysql::params;
                        let delete_stmt = String::from(#delete_statement);
                        let params= mysql::params!{ #(#alter_delete_params),* };
                        toql::log::info!("Sql `{}` with params {:?}", delete_stmt, params);

                        let mut stmt = conn.prepare(delete_stmt)?;
                        let res = stmt.execute(params)?;
                        Ok(res.affected_rows())
                     }

                        fn delete_many<I>(entities: I, conn: &mut mysql::Conn) -> Result<u64, toql::error::ToqlError>
                        where I:  Iterator<Item=&'a #struct_ident>
                        {
                             use mysql::params;
                            let mut delete_stmt = String::from(#delete_many_statement);

                            let mut params :Vec<(String, String)>= Vec::new();

                            for (i, entity) in entities.enumerate() {
                                    delete_stmt.push('(');
                                   #(#alter_delete_key_comparison)*
                                   delete_stmt.push(')');
                                   delete_stmt.push_str( " OR ");
                                   #(#alter_delete_many_params);*
                            }
                            if params.is_empty() {return Ok(0);}

                            let delete_stmt = delete_stmt.trim_end_matches(" OR "); // Let as &str
                            toql::log::info!("Sql `{}` with params {:?}", delete_stmt, params);
                            let mut stmt = conn.prepare(delete_stmt)?;
                            let res= stmt.execute(params)?;
                            Ok(res.affected_rows())

                     }

                }

            }
        };
        
        log::debug!("Source code for `{}`:\n{}",self.struct_ident, alter.to_string());
        tokens.extend(alter);
    }   
    
}