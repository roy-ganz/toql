use proc_macro2::{TokenStream};
use syn::Ident;



use darling::{Error, Result};

use crate::sane::{FieldKind, SqlTarget};

pub(crate) struct GeneratedMysqlSelect<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    sql_table_alias: String,

    select_columns: Vec<String>,
    select_columns_params: Vec<TokenStream>,

    select_joins: Vec<String>,
    select_joins_params: Vec<TokenStream>,

    select_keys: Vec<TokenStream>,

    select_keys_params: Vec<TokenStream>,

   

    merge_fields: Vec<crate::sane::Field>,
    merge_self_fields: Vec<String>,
}

impl<'a> GeneratedMysqlSelect<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlSelect {
        GeneratedMysqlSelect {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            sql_table_alias: toql.sql_table_alias.to_owned(),

            select_columns: Vec::new(),
            select_columns_params: Vec::new(),

            select_joins: Vec::new(),
            select_joins_params: Vec::new(),

            select_keys: Vec::new(),

            select_keys_params: Vec::new(),

          
            merge_fields: Vec::new(),
            merge_self_fields: Vec::new(),
        }
    }

    pub fn add_select_field(&mut self, field: &crate::sane::Field) -> Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if regular_attrs.key {
                    let key_index = syn::Index::from(self.select_keys.len());
                    if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                        let key_expr = format!("{}.{} = ?", self.sql_table_alias, sql_column);
                        self.select_keys.push(quote!(#key_expr));
                    } else {
                            return Err(Error::custom(
                                "SQL expression not allowed for key. Remove `sql` from #[toql(..)]",
                            ));
                    }

                    self.select_keys_params.push(quote! {
                        params.push( key . #key_index .to_string());
                    });

                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        // Check if sql expression uses aux params.
                        // Aux params can only be used with load methods (toql query and mapper)
                        // Sql expressions with aux params must be selectable and will load always NULL
                        
                        if field.number_of_options > 0 && !field.preselect {
                                self.select_columns.push(String::from("NULL"));
                                } else {
                                    return Err(Error::custom(
                                    "SQL expression cannot be selected. Either make field selectable with `Option<..>` so it can be `None` or skip selection by adding the attribute `#[toql(skip_select)]` to the struct",
                                ).with_span(&rust_type_ident));
                        }      
                    }
                    SqlTarget::Column(ref sql_column) => {
                        self.select_columns
                            .push(format!("{{alias}}.{}", sql_column));
                    }
                }
             }
            FieldKind::Join(ref join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;
                let alias = &join_attrs.join_alias;
                

                // Don't select join, if on clause has on_handler or uses parameters <..> 
                /* if let Some(s) = &join_attrs.on_sql {
                    let regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
                    if regex.is_match(s) {
                         if field.number_of_options > 1
                            || (field.number_of_options == 1 && field.preselect == true)
                        {


                        }

                } */

                //if join_attrs. TODO Skip for on_handler 
                
            

                // Add discriminatory field for left join
                 if field.number_of_options > 1
                    || (field.number_of_options == 1 && field.preselect == true)
                {
                    let none_format = format!("{}.{{}} IS NOT NULL", &join_attrs.join_alias);
                    let none_condition = quote!(<<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().map(|other_column|{
                                                format!(#none_format,  &other_column)
                                        }).collect::<Vec<String>>().join(" AND "));

                    self.select_columns.push(String::from("{}"));
                    self.select_columns_params.push(none_condition);
                }

                self.select_columns.push(String::from("{}"));
                self.select_columns_params.push(
                    quote!( <Self as toql::select::Select<#rust_type_ident>>:: columns_sql(#alias)),
                );

                if join_attrs.key {
                    let key_index = syn::Index::from(self.select_keys.len());
                    let aliased_column_format = format!("{}.{{}} = ?", &self.sql_table_alias);
                    self.select_keys.push(quote!( {
                        <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter()
                        .map(|other_column|{
                            #default_self_column_code;
                            let self_column = #columns_map_code;
                            format!(#aliased_column_format, self_column )
                        }).collect::<Vec<String>>().join(" AND ").as_ref()
                    }
                    ));
                    self.select_keys_params.push(  quote! {
                            params.extend_from_slice( &toql::key::Key::params( &key. #key_index));
                        });
                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                

                let join_type = if field.number_of_options == 0
                    || (field.number_of_options == 1 && field.preselect == false)
                {
                    ""
                } else {
                    "LEFT "
                };

                self.select_joins.push(format!(
                    "{}JOIN {} {} {{}}ON ({{}}{{}})",
                    join_type, join_attrs.sql_join_table_name, &join_attrs.join_alias
                ));

                self.select_joins_params
                    .push(quote!(<Self as toql::select::Select<#rust_type_ident>> :: joins_sql()));

                let select_join_params_format = format!(
                    "{}.{{}} = {}.{{}}",
                    &self.sql_table_alias, join_attrs.join_alias
                );
                self.select_joins_params.push(quote!(
                    {


                      <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter()

                        .map(|other_column| {
                            #default_self_column_code;
                            let self_column= #columns_map_code;

                        format!(#select_join_params_format, self_column, other_column)
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                ));
                self.select_joins_params
                    .push(if let Some(ref sql) = &join_attrs.on_sql {
                        let on_sql = format!(
                            " AND ({})",
                            sql.replace("..", &format!("{}.", join_attrs.join_alias))
                        );
                        quote!( #on_sql)
                    } else {
                        quote!("")
                    });
            }
            FieldKind::Merge(_) => {
                self.merge_fields.push(field.clone());
            }
        };
        Ok(())
    }

   
}

impl<'a> quote::ToTokens for GeneratedMysqlSelect<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let sql_table_name = &self.sql_table_name;
        let sql_table_alias = &self.sql_table_alias;

        let select_columns = self.select_columns.join(", ");

        let select_columns_params = &self.select_columns_params;
        let select_joins = self.select_joins.join(" ");

        let select_joins_params = &self.select_joins_params;

        let select_statement = format!(
            "SELECT {{}} FROM {} {} {{}}{{}}",
            sql_table_name, sql_table_alias
        );

        let columns_sql_code = if select_columns_params.is_empty() {
            quote!(format!(#select_columns, alias=alias))
        } else {
            quote!(format!(#select_columns, #(#select_columns_params),*, alias = alias))
        };
        let joins_sql_code = if select_joins_params.is_empty() {
            quote!( String::from(#select_joins))
        } else {
            quote!(format!(#select_joins, #(#select_joins_params),*))
        };

        let select = quote! {
                       impl<'a, T: toql::mysql::mysql::prelude::GenericConnection + 'a> toql::select::Select<#struct_ident> for toql::mysql::MySql<'a,T> {

                           type Error = toql :: mysql::error:: ToqlMySqlError;

                            fn table_alias() -> String {
                                String::from(#sql_table_alias)
                            }

                           fn columns_sql(alias: &str) -> String {
                                  #columns_sql_code

                           }
                           fn joins_sql() -> String {
                                   #joins_sql_code
                           }
                           fn select_sql(join: Option<&str>) -> String {
                                   format!( #select_statement,
                                   <Self as  toql::select::Select<#struct_ident>>::columns_sql(#sql_table_alias),
                                   <Self as  toql::select::Select<#struct_ident>>::joins_sql(),
                                   join.unwrap_or(""))
                           }


                            fn select_one(&mut self, key: <#struct_ident as toql::key::Keyed>::Key)
                            -> Result<#struct_ident, toql :: mysql::error:: ToqlMySqlError>
                            {
                              let conn = self.conn();

                               //let (predicate, params) = toql::key::predicate_sql::<#struct_ident,_>( &[key], Some(#sql_table_alias));
                               let (predicate, params) =  <<#struct_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_predicate(&key, #sql_table_alias);
                               let select_stmt = format!(
                                   "{} WHERE {} LIMIT 0,2",
                                   <Self as toql::select::Select<#struct_ident>>::select_sql(None),
                                   predicate
                               );

                               toql::log_sql!(select_stmt, params);

                               let entities_stmt = conn.prep_exec(select_stmt, &params)?;
                               let mut entities = toql::mysql::row::from_query_result::< #struct_ident>(entities_stmt)?;

                               if entities.len() > 1 {
                                   return Err( toql::mysql::error::ToqlMySqlError::ToqlError( toql::error::ToqlError::NotUnique));
                               } else if entities.is_empty() {
                                   return Err( toql::mysql::error::ToqlMySqlError::ToqlError( toql::error::ToqlError::NotFound));
                               }

                               Ok(entities.pop().unwrap())
                            }
                             fn select_many(&mut self, keys: &[<#struct_ident as toql::key::Keyed>::Key]) -> Result<Vec<#struct_ident>, Self::Error>{

                                   let conn = self.conn();
                                   
                                   //let (predicate, params) = toql::key::predicate_sql::<#struct_ident,_>( keys, Some(#sql_table_alias));
                                   let (predicate, params) = <<#struct_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_any_predicate(keys, #sql_table_alias);

                                    let select_stmt = format!(
                                   "{} WHERE {}",
                                   <Self as toql::select::Select<#struct_ident>>::select_sql(None),
                                   predicate,
                               );

                               toql::log_sql!(select_stmt, params);
                               let entities_stmt = conn.prep_exec(select_stmt, &params)?;
                               let entities = toql::mysql::row::from_query_result::< #struct_ident>(entities_stmt)?;
                               Ok(entities)
                             }

                              /*  fn select_dependencies( &mut self, join: &str, params:&Vec<String>) -> Result<Vec<#struct_ident> , toql :: mysql::error:: ToqlMySqlError>

                                   {
                                       let conn = self.conn();
                                       let select_stmt =  <Self as  toql::select::Select<#struct_ident>>::select_sql(Some(join));

                               toql::log_sql!(select_stmt, params);

                               let entities_stmt = conn.prep_exec(select_stmt, params)?;
                               let mut entities = toql::mysql::row::from_query_result::<#struct_ident>(entities_stmt)?;


                               let key_predicate = [ #(#select_keys),*].join( " AND ");
                             //  #(#merge_code)* TODO merge

                               Ok(entities)
                               }
        */
                       }

               };

        log::debug!(
            "Source code for `{}`:\n{}",
            self.struct_ident,
            select.to_string()
        );
        tokens.extend(select);
    }
}
