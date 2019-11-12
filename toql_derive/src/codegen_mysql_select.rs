use crate::annot::Toql;
use crate::annot::ToqlField;

use proc_macro2::Span;
use syn::Ident;

use heck::MixedCase;
use heck::SnakeCase;

use darling::{Result, Error};

use crate::sane::{FieldKind, JoinField, MergeField, RegularField, SqlTarget};

pub(crate) struct GeneratedMysqlSelect<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    sql_table_alias: String,
    vis: &'a syn::Visibility,

    select_columns: Vec<String>,
    select_columns_params: Vec<proc_macro2::TokenStream>,

    select_joins: Vec<String>,
    select_joins_params: Vec<proc_macro2::TokenStream>,

    select_keys: Vec<proc_macro2::TokenStream>,

    select_key_types: Vec<proc_macro2::TokenStream>,
    select_key_fields: Vec<proc_macro2::TokenStream>,
    select_keys_params: Vec<proc_macro2::TokenStream>,
    key_predicates: Vec<proc_macro2::TokenStream>,

    key_setters: Vec<proc_macro2::TokenStream>,

    merge_code: Vec<proc_macro2::TokenStream>,
    key_columns_code: Vec<proc_macro2::TokenStream>,
    key_params_code: Vec<proc_macro2::TokenStream>,

    merge_fields: Vec<crate::sane::Field>,
    merge_self_fields: Vec<String>,
}

impl<'a> GeneratedMysqlSelect<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlSelect {
        GeneratedMysqlSelect {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            sql_table_alias: toql.sql_table_alias.to_owned(),
            vis: &toql.rust_struct_visibility,

            select_columns: Vec::new(),
            select_columns_params: Vec::new(),

            select_joins: Vec::new(),
            select_joins_params: Vec::new(),

            select_keys: Vec::new(),

            select_key_types: Vec::new(),
            select_key_fields: Vec::new(),
            select_keys_params: Vec::new(),
            key_predicates: Vec::new(),
            key_setters: Vec::new(),

            merge_code: Vec::new(),
            key_columns_code: Vec::new(),
            key_params_code: Vec::new(),
            merge_fields: Vec::new(),
            merge_self_fields: Vec::new(),
        }
    }

    pub fn add_select_field(&mut self, field: &crate::sane::Field) ->Result<()> {
        /*   let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ident = field.ident.as_ref().unwrap();
        let sql_column = crate::util::rename(&field_ident.to_string(), &toql.columns);
        let sql_table_name = &self.sql_table_ident.to_string();
        let sql_table_alias = sql_table_name.to_snake_case();
        let field_type = field.first_non_generic_type().unwrap(); */

        let rust_type_ident = &field.rust_type_ident;
        let rust_type_name = &field.rust_type_name;
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let toql_field_name = &field.toql_field_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if regular_attrs.key {
                   /*  if let SqlTarget::Column(ref column) = &regular_attrs.sql_target {
                        self.key_columns_code
                            .push(quote!( columns.push( String::from(#column)); ));
                    } else {
                        // error only
                    }

                    self.select_key_types.push(quote!( #rust_type_ident));

                    if field.number_of_options > 0 {
                        let value = quote!(self. #rust_field_ident .as_ref() .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))? .to_owned());
                        self.select_key_fields.push(value);

                        /*  self.key_params_code.push( quote!(
                        params.push(self. #rust_field_ident .as_ref()
                         .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_field_name)))? .to_owned().to_string());
                         )); */

                        let index = syn::Index::from(self.select_key_types.len() - 1);
                        self.key_setters
                            .push(quote!(self. #rust_field_ident = Some( key . #index  ); ))
                    } else {
                        self.select_key_fields
                            .push(quote!(self. #rust_field_ident .to_owned()));

                        let index = syn::Index::from(self.select_key_types.len() - 1);

                        self.key_setters
                            .push(quote!(self. #rust_field_ident = key . #index;))
                    }

                    if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                        let key_expr = format!("{}.{} = ?", self.sql_table_alias, sql_column);
                        self.select_keys.push(quote!(#key_expr));
                    } else {
                        // error only
                    }

                    let key_index = syn::Index::from(self.select_key_fields.len() - 1);
                    self.key_predicates.push(quote! {
                     .and(toql::query::Field::from(#toql_field_name).eq(&key . #key_index))
                    });

                    self.key_params_code
                        .push(quote!(params.push(key . #key_index .to_owned().to_string()); ));

                    self.select_keys_params.push(quote! {
                     params.push( key . #key_index .to_string());
                    }); */

                     let key_index = syn::Index::from(self.select_keys.len());
                    if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                        let key_expr = format!("{}.{} = ?", self.sql_table_alias, sql_column);
                        self.select_keys.push(quote!(#key_expr));
                    } else {
                        return Err(Error::custom("SQL expression not allowed for key. Remove `sql` from #[toql(..)]"));
                    }
                     
                    self.select_keys_params.push(quote! {
                        params.push( key . #key_index .to_string());
                    });

                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                // Normal key should may only one Option (Toql selectable)
                /* self.select_keys_params.push( match field.number_of_options() {
                    1 => quote!( params.push( key
                                .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                .to_string()
                                ); ),
                    0 => quote!( params.push( key .to_string()); ),
                    _ => unreachable!()
                }
                );  */

                match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        // TODO 
                    }
                    SqlTarget::Column(ref sql_column) => {
                        self.select_columns
                            .push(format!("{}.{}", &self.sql_table_alias, sql_column));
                    }
                };
            }
            FieldKind::Join(ref join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;

                self.select_columns.push(String::from("true"));
                self.select_columns.push(String::from("{}"));
                self.select_columns_params
                    .push(quote!(#rust_type_ident :: columns_sql()));

                if join_attrs.key {
                  /*   self.select_key_types
                        .push(quote!( <#rust_type_ident as toql::key::Key>::Key));
                    let key_index = syn::Index::from(self.select_key_types.len() - 1);

                    /*  let unwrap = match field.number_of_options {
                        1 if !field.preselect => quote!(.as_ref().unwrap()),
                        _ => quote!()

                    }; */

                    self.key_columns_code.push( quote!( columns.extend_from_slice(&<#rust_type_ident as toql::key::Key>::columns());));
                    self.key_params_code.push( quote!( params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params(& key. #key_index));));

                    //let function_ident = Ident::new(&format!("{}_key_predicate",&field.rust_field_name), Span::call_site());

                    self.key_predicates.push(
                        quote!(
                                .and( <#rust_type_ident as toql::key::Key>::Key::key_predicate(&key. #key_index)?)
                            )
                    );
                    // Select key predicate
                    if field.number_of_options > 0 {
                        self.select_key_fields.push( quote!(
                                < #rust_type_ident as toql::key::Key>::get_key(
                                    self. #rust_field_ident .as_ref()
                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))?
                                    )?
                            ));

                        self.key_setters.push( quote!(
                                        < #rust_type_ident as toql::key::Key>::set_key(self. #rust_field_ident .as_mut()
                                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))? , key . #key_index )?;
                            ));
                    } else {
                        self.select_key_fields.push( quote!(
                                < #rust_type_ident as toql::key::Key>::get_key(  &self. #rust_field_ident )?
                            ));

                        self.key_setters.push( quote!(
                                    < #rust_type_ident as toql::key::Key>::set_key(&mut self. #rust_field_ident,key . #key_index)?;
                            ));
                    }
                    let aliased_column_format = format!("{}.{{}} = ?", &self.sql_table_alias);
                    self.select_keys.push(quote!( {
                        &<#rust_type_ident as toql::key::Key>::columns().iter()
                        .map(|other_column|{
                            #default_self_column_code;
                            let self_column = #columns_map_code;
                            format!(#aliased_column_format, self_column )
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                    ));
                    self.select_keys_params.push(  quote! {
                            params.extend_from_slice( &<#rust_type_ident as toql::key::Key>::params( &key. #key_index));
                        });
 */
                    let key_index = syn::Index::from(self.select_keys.len());
                    let aliased_column_format = format!("{}.{{}} = ?", &self.sql_table_alias);
                    self.select_keys.push(quote!( {
                        &<#rust_type_ident as toql::key::Key>::columns().iter()
                        .map(|other_column|{
                            #default_self_column_code;
                            let self_column = #columns_map_code;
                            format!(#aliased_column_format, self_column )
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                    ));
                    self.select_keys_params.push(  quote! {
                            params.extend_from_slice( &<#rust_type_ident as toql::key::Key>::params( &key. #key_index));
                        });
                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                self.select_joins.push(format!(
                    "JOIN {} {} ON ({{}}{{}}) {{}}",
                    join_attrs.sql_join_table_name, rust_field_name
                ));

                let select_join_params_format = format!(
                    "{}.{{}} = {}.{{}}",
                    &self.sql_table_alias, join_attrs.join_alias
                );

                self.select_joins_params.push(quote!(
                    {


                      <#rust_type_ident as toql::key::Key>::columns().iter()

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

                self.select_joins_params
                    .push(quote!(#rust_type_ident :: joins_sql()));
            }
            FieldKind::Merge(_) => {
                self.merge_fields.push(field.clone());
            }
        };

        // Regular field
        /* if field.join.is_none() && field.merge. is_none() {
                if field.key == true {


                     self.key_columns_code.push( quote!( columns.push( String::from(#sql_column)); ));


                    let key_type = field.first_non_generic_type();
                    self.select_key_types.push(quote!( #key_type));

                   if field.number_of_options() > 0 {
                       let value= quote!(self. #field_ident .as_ref() .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? .to_owned());
                       self.select_key_fields.push( value);

                       self.key_params_code.push( quote!(
                           params.push(self. #field_ident .as_ref()
                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? .to_owned().to_string());
                            ));


                    let index =  syn::Index::from(self.select_key_types.len()-1);
                    self.key_setters.push( quote!(self. #field_ident = Some( key . #index  ); ))
                   } else {

                    self.select_key_fields.push( quote!(self. #field_ident .to_owned()));
                     self.key_params_code.push( quote!(params.push(self. #field_ident .to_owned().to_string());
                     ));


                     let index =  syn::Index::from(self.select_key_types.len()-1);

                     self.key_setters.push( quote!(self. #field_ident = key . #index;) )
                   }

                    let key_expr = format!("{}.{} = ?",sql_table_alias, sql_column);
                    self.select_keys.push(quote!(#key_expr));

                  let toql_field = field_name.to_mixed_case();
                  let key_index= syn::Index::from(self.select_key_fields.len() - 1);
                  self.key_predicates.push(  quote! {
                     .and(toql::query::Field::from(#toql_field).eq( key . #key_index))
                    });
                  self.select_keys_params.push(  quote! {
                     params.push( key . #key_index .to_string());
                    });




                         // Normal key should may only one Option (Toql selectable)
                     /* self.select_keys_params.push( match field.number_of_options() {
                        1 => quote!( params.push( key
                                    .ok_or(toql::error::ToqlError::ValueMissing(String::from(#field_name)))?
                                    .to_string()
                                    ); ),
                        0 => quote!( params.push( key .to_string()); ),
                        _ => unreachable!()
                    }
                    );  */


                }
                self.select_columns.push(format!("{}.{}",sql_table_alias, sql_column));
            }
            // Join field
           /*  else */ if field.join.is_some() {

                 /* Joins can also be fields.
                    The key_type and key field on the joined struct must be provided
                 */


                 self.select_columns.push(String::from("true"));
                 self.select_columns.push(String::from("{}"));
                 self.select_columns_params.push( quote!(#field_type :: columns_sql()));

                let sql_join_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
                let default_join_alias = sql_join_table_name.to_snake_case();




                let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

              //  let mut on_condition: Vec<String>= Vec::new();


               // let default_self_columns= vec![crate::util::rename(&format!("{}_id", field_name), &toql.columns)];
             /*    let self_columns =  if !field.join.as_ref().unwrap().this_columns.is_empty() {
                    field.join.as_ref().unwrap().this_columns.as_ref() }
                    else {
                        &default_self_columns
                    };
                    self.key_columns_code.push( quote!( columns.extend_from_slice(&<#field_type as toql::key::Key>::columns());));

                    let default_other_columns= vec![crate::util::rename("id", &toql.columns)];
                let other_columns =  if !field.join.as_ref().unwrap().other_columns.is_empty() {
                    field.join.as_ref().unwrap().other_columns.as_ref() }
                    else {
                        &default_other_columns
                    }; */

                    // Toql key predicate
                    //let toql_field = format!("{}_{}",field_name.to_mixed_case(), other_column.to_string().to_mixed_case());

                     if field.key == true {
                        self.select_key_types.push(quote!( <#field_type as toql::key::Key>::Key));
                        let key_index= syn::Index::from(self.select_key_types.len() -1);


                        self.key_columns_code.push( quote!( columns.extend_from_slice(&<#field_type as toql::key::Key>::columns());));
                        self.key_params_code.push( quote!( params.extend_from_slice(&<#field_type as toql::key::Key>::params(self. #field_ident)?);));

                        //let join_key_index= syn::Index::from(i);
                        let toql_format_template = format!("{}_{{}}",field_name.to_mixed_case());
                        self.key_predicates.push(
                            quote!(
                                .and(
                                    {
                                        let q = toql::query::Query::new();
                                        for (other_column, param) in < #field_type as toql::key::Key>:: columns().iter().zip(< #field_type as toql::key::Key>:: params())
                                        {
                                            let toql_field =  format!(#toql_format_template, other_column.to_owned().to_mixed_case());
                                            q.and(toql::query::Field::from(toql_field).eq ( param.to_owned()))
                                        }
                                        q
                                    }
                                )
                            )
                        );
                        // Select key predicate
                        if field.number_of_options() > 0 {
                                self.select_key_fields.push( quote!(
                                    < #field_type as toql::key::Key>::get_key(
                                        self. #field_ident .as_ref()
                                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))?
                                        )?
                                ));



                                    self.key_setters.push( quote!(
                                            < #field_type as toql::key::Key>::set_key(self. #field_ident .as_mut()
                                                .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? , key . #key_index );
                                ));

                            } else {
                                self.select_key_fields.push( quote!(
                                    < #field_type as toql::key::Key>::get_key(  &self. #field_ident )?
                                ));

                                self.key_setters.push( quote!(
                                        < #field_type as toql::key::Key>::set_key(&mut self. #field_ident,key . #key_index);
                                ));

                                let default_column_template = format!("{}_{{}}", join_alias);
                                let column_translation = field.join.as_ref().unwrap().columns.iter()
                                    .map(|column| {
                                        let tc = &column.this; let oc = &column.other;
                                        quote!(oc => tc,)  })
                                    .collect::<Vec<_>>();


                                self.select_keys.push(
                                    quote!( {

                                        #field_type as toql::key::Key>::columns().as_ref()
                                        .map(|other_column|{
                                            let default_self_column= format!(#default_column_template, self_column);
                                            let self_column = match other_column.as_str(){
                                                    #(#column_translation)*
                                                    _ => &default_elf_column
                                            };
                                            format!("{}.{} = ?", sql_table_alias,self_column );
                                        }).collect::<Vec<String>>().join(" AND ");
                                    }
                                    ));
                                self.select_keys_params.push(  quote! {
                                    <#field_type as toql::key::Key>::push_values(&params);
                                    });

                            }
                }

                       /*  let default_column_format = format!("{}_{{}}", field_ident);
                        let match_translation :String = field.join.as_ref().unwrap().columns.iter()
                            .map(|column| { format!("{} => {},", column.this, column.other)  })
                            .collect::<Vec<String>>()
                            .join("");

                       on_condition.push(
                           quote!(
                                #field_type as toql::key::Key>::columns().as_ref().iter()
                                .map(|other_column| {
                                    let default_self_column = format!(#default_column_format, other_column);
                                    let self_column = match other_column {
                                        #match_translation
                                        _ => default_self_column
                                    }
                                    format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias, other_column )
                                }).collect::<Vec<String>>().join(" AND "))
                           )
                        ); */



                /* self_columns.iter().zip(other_columns).enumerate().for_each( |(i, (self_column, other_column))| {

                    if field.key == true {

                        // Join always on key

                        let struct_key_type = Ident::new(&format!("{}Key", &field_type), Span::call_site());
                        self.select_key_types.push(quote!( <#field_type as toql::key::Key>::Key));


                           /* let toql_field = format!("{}_{}",field_name.to_mixed_case(), other_column.to_string().to_mixed_case());
                           let key_index= syn::Index::from(self.select_key_types.len() -1);
                           let join_key_index= syn::Index::from(i);
                            self.key_predicates.push(  quote! {
                            .and(toql::query::Field::from(#toql_field).eq ((key . #key_index) .#join_key_index ))
                            }); */

                        if field.number_of_options() > 0 {
                            self.select_key_fields.push( quote!(
                                < #field_type as toql::key::Key>::get_key(
                                    self. #field_ident .as_ref()
                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))?
                                    )?
                            ));

                            let index =  syn::Index::from(self.select_key_types.len()-1);
                              self.key_setters.push( quote!(
                                < #field_type as toql::key::Key>::set_key(self. #field_ident .as_mut()
                                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(# field_name)))? , key . #index );
                            ));

                        } else {
                             self.select_key_fields.push( quote!(
                                < #field_type as toql::key::Key>::get_key(  &self. #field_ident )?
                            ));

                           // let index =  syn::Index::from(self.select_key_types.len()-1);
                              self.key_setters.push( quote!(
                                < #field_type as toql::key::Key>::set_key(&mut self. #field_ident,key . #key_index);
                            ));

                             self.select_keys.push(format!("{}.{} = ?", sql_table_alias,self_column ));
                             self.select_keys_params.push(  quote! {
                                params.push( (key . #key_index) . #join_key_index .to_string());
                                });

                        }



                 }  */



                   // on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias, other_column )); on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias, other_column ));

                    // TODO custom on clause
        //        });



                // TODO rename join entity
               // let join_statement = format!("JOIN {} {} ON ({{}} AND ({}))",sql_join_table_name, field_name,  on_condition.join(" AND "));
               /*  self.select_joins_code.push(quote!(

                      let join_expression = <Language as toql::key::Key>::columns().iter().zip(&[ #(#self_columns),* ]).map(|(other_column, self_column)| {
                        format!("{} = {}.{}",sql_alias , if sql_alias.is_empty() { "" } else { "." }, self_column, #join_alias, other_column)
                        }).collect::<Vec<String>>().join(" AND ");

                    format!(#join_statement, join_expression )

                )); */

                 self.select_joins.push(format!("JOIN {} {} ON ({{}}{{}}) {{}}",sql_join_table_name, field_name));


                let select_join_params_format= format!("{}.{{}} = {}.{{}}",sql_table_alias, join_alias);
                 let default_column_format = format!("{}_{{}}", field_ident);
                  let match_translation  = field.join.as_ref().unwrap().columns.iter()
                            .map(|column| {
                                let tc = &column.this; let oc = &column.other;
                                quote!(#oc => #tc, )
                                })
                            .collect::<Vec<_>>();
                self.select_joins_params.push( quote!(
                    {


                      <Language as toql::key::Key>::columns().iter()

                        .map(|other_column| {
                            let default_column= format!(#default_column_format, other_column);
                            let self_column= match other_column.as_str() {
                                #(#match_translation)*
                                _ => &default_column
                            };

                        //format!("{}{}{} = {}.{}",sql_alias , if sql_alias.is_empty() { "" } else { "." }, self_column, #join_alias, other_column)
                        format!(#select_join_params_format, self_column, other_column)
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                ));
                self.select_joins_params.push( if field.join.as_ref().unwrap().on_sql.is_some() {
                        let on_sql= format!(" AND ({})", &field.join.as_ref().unwrap().on_sql.as_ref().unwrap().replace("..",&format!("{}.",join_alias)));
                            quote!( #on_sql)
                    } else {
                        quote!()});
                 self.select_joins_params.push( quote!(#field_type :: joins_sql()));
            }
            // Merge field
            else */
        Ok(())
    }
   
    pub fn build_merge(&mut self) {
        // Build all merge fields
        // This must be done after the first pass, becuase all key names must be known at this point
        for field in &self.merge_fields {
            match &field.kind {
                FieldKind::Merge(merge_attrs) => {
                    let mut on_condition: Vec<String> = Vec::new();
                    if let Some(sql) = &merge_attrs.on_sql {
                        // resolve ..
                        on_condition.push(
                            sql.replace("..", &format!("{}.", merge_attrs.join_alias).to_owned()),
                        );
                    }

                    // Build join for all keys of that struct
                    for self_field in &self.merge_self_fields {
                        let default_other_field =
                            format!("{}_{}", field.rust_type_name.to_mixed_case(), &self_field);
                        let other_field = merge_attrs.other_field(&self_field, default_other_field);

                        let self_column = merge_attrs.column(&self_field);
                        let other_column = merge_attrs.column(&other_field);

                        on_condition.push(format!(
                            "{}.{} = {}.{}",
                            merge_attrs.join_alias,
                            self_column,
                            merge_attrs.join_alias,
                            other_column
                        ));
                    }

                    let merge_join = format!(
                        "JOIN {} {} ON ({} AND {{}})",
                        merge_attrs.sql_join_table_name,
                        merge_attrs.join_alias,
                        on_condition.join(" AND ")
                    );

                    let struct_ident = self.struct_ident;
                    let merge_function = Ident::new(
                        &format!("merge_{}", &field.rust_field_name),
                        Span::call_site(),
                    );

                    let merge_field_init = if field.number_of_options > 0 {
                        quote!(Some(Vec::new()))
                    } else {
                        quote!(Vec::new())
                    };
                    let rust_field_ident = &field.rust_field_ident;
                    let rust_type_ident = &field.rust_type_ident;

                    self.merge_code.push(quote!(
                                let #rust_field_ident = #rust_type_ident :: select_dependencies( &format!(#merge_join, key_predicate), &params, conn)?;
                                for e in entities.iter_mut() { e . #rust_field_ident = #merge_field_init; }
                                #struct_ident :: #merge_function(&mut entities, #rust_field_ident);
                        ));
                }
                _ => {
                    panic!("Should be never called!");
                }
            }
        }
    }
}

/* fn build_merge(&self) {
{

            let sql_join_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
             let default_join_alias = sql_join_table_name.to_snake_case();


            let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

             let mut on_condition: Vec<String>= Vec::new();
            for j in &field.merge {
                //let auto_self_key : String = crate::util::rename(&field_ident.to_string(), &toql.columns);
                let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
                let auto_self_field= "id".to_string();

                let self_column :String = crate::util::rename(&j.this_field.as_ref().unwrap_or(&auto_self_field).to_string(), &toql.columns);

                let other_column = crate::util::rename(&j.other_field.as_ref().unwrap_or(&auto_other_field).to_string(), &toql.columns);
                on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias,other_column, ));
            }


            let merge_field_init = if field.number_of_options() > 0 {
                 quote!( Some(Vec::new()))
            } else {
                quote!(Vec::new())
            };


            let dep_join = format!("JOIN {} {} ON ({} AND {{}})", sql_table_name, sql_table_alias, on_condition.join(" AND "));

            let struct_ident = self.struct_ident;
            let merge_function = Ident::new( &format!("merge_{}", &field.ident.as_ref().unwrap()),  Span::call_site());


            self.merge_code.push(quote!(
                    let #field_ident = #field_type :: select_dependencies( &format!(#dep_join, key_predicate), &params, conn)?;
                     for e in entities.iter_mut() { e . #field_ident = #merge_field_init; }
                    #struct_ident :: #merge_function(&mut entities, #field_ident);
            ));

        }

} */

/* let sql_join_table_name = crate::util::rename(&field_type.to_string(), &toql.tables);
 let default_join_alias = sql_join_table_name.to_snake_case();


let join_alias = &field.alias.as_ref().unwrap_or(&default_join_alias);

 let mut on_condition: Vec<String>= Vec::new();
for j in &field.merge {
    //let auto_self_key : String = crate::util::rename(&field_ident.to_string(), &toql.columns);
    let auto_other_field= format!("{}_id", self.struct_ident.to_string().to_snake_case());
    let auto_self_field= "id".to_string();

    let self_column :String = crate::util::rename(&j.this_field.as_ref().unwrap_or(&auto_self_field).to_string(), &toql.columns);

    let other_column = crate::util::rename(&j.other_field.as_ref().unwrap_or(&auto_other_field).to_string(), &toql.columns);
    on_condition.push(format!("{}.{} = {}.{}",sql_table_alias, self_column, join_alias,other_column, ));
}


let merge_field_init = if field.number_of_options() > 0 {
     quote!( Some(Vec::new()))
} else {
    quote!(Vec::new())
};


let dep_join = format!("JOIN {} {} ON ({} AND {{}})", sql_table_name, sql_table_alias, on_condition.join(" AND "));

let struct_ident = self.struct_ident;
let merge_function = Ident::new( &format!("merge_{}", &field.ident.as_ref().unwrap()),  Span::call_site());


self.merge_code.push(quote!(
        let #field_ident = #field_type :: select_dependencies( &format!(#dep_join, key_predicate), &params, conn)?;
         for e in entities.iter_mut() { e . #field_ident = #merge_field_init; }
        #struct_ident :: #merge_function(&mut entities, #field_ident);
));  */

// }
impl<'a> quote::ToTokens for GeneratedMysqlSelect<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let sql_table_name = &self.sql_table_name;
        let sql_table_alias = &self.sql_table_alias;

        let key_types = &self.select_key_types;

        let key_type_code = quote!(  #(pub #key_types),* );

        //  let key_params_code = &self.key_params_code;

        
            let select_columns = self.select_columns.join(",");

            let select_columns_params = &self.select_columns_params;
            let select_joins = self.select_joins.join(" ");

            let select_joins_params = &self.select_joins_params;

            // let select_keys_params= &self.select_keys_params;

            let select_statement = format!(
                "SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name, sql_table_alias
            );

            let select_stmt = format!(
                "SELECT {{}} FROM {} {} {{}}WHERE {{}}",
                sql_table_name, sql_table_alias
            );
            let select_one_stmt = format!("{} LIMIT 0,2", select_stmt);

            let select_dependend_stmt = format!(
                "SELECT {{}} FROM {} {} {{}}{{}}",
                sql_table_name, sql_table_alias
            );

            let merge_code = &self.merge_code;

            //let merge_key_predicate = self.select_keys.join(" AND ");

            let select_keys = &self.select_keys;
            let select_keys_params = &self.select_keys_params; /* Vec<proc_macro2::TokenStream> = self.select_key_types.iter().enumerate().map(|x| {
                                                               let i = x.0;
                                                               let is = syn::Index::from(i);
                                                               quote!(params.push(key. #is .to_string()); )} ).collect(); */

            let columns_sql_code = if select_columns_params.is_empty() {
                quote!( String::from(#select_columns))
            } else {
                quote!(format!(#select_columns, #(#select_columns_params),*))
            };
            let joins_sql_code = if select_joins_params.is_empty() {
                quote!( String::from(#select_joins))
            } else {
                quote!(format!(#select_joins, #(#select_joins_params),*))
            };

            let select_key_fields = &self.select_key_fields;
            // let key_setters_fragments =  &self.key_setters;

            let key_getter = quote!( #(#select_key_fields  ),* );

            /*
            let key_setters = if  key_setters_fragments.len() == 1 {
                vec![quote!( #(#key_setters_fragments ;)* )];
            } else {
                key_setters_fragments.iter().enumerate().map(|x| {
                                     let s = x.1;
                                     let i = x.0;
                                     let is = syn::Index::from(i);
                                     quote!(#s . #is; )} ).collect();

            };  */

            let key_predicates = &self.key_predicates;
            /* self.key_predicates.iter().enumerate().map(|x| {
            let s = x.1;
            let i = x.0;
            let is = syn::Index::from(i);
            quote!( .and(#s.eq( key . #is)))  }).collect(); */

            let key_setters = &self.key_setters;
            /* Vec<proc_macro2::TokenStream> =
            self.key_setters.iter().enumerate().map(|x| {
                                let s = x.1;
                                let i = x.0;
                                let is = syn::Index::from(i);
                                quote!( #s = key . #is;)  }).collect(); */

            let key_predicate_fn = Ident::new(
                &format!("{}_key_predicate", &struct_ident).to_snake_case(),
                Span::call_site(),
            );
            let vis = self.vis;

            let struct_key_ident = Ident::new(&format!("{}Key", &struct_ident), Span::call_site());
            let key_columns_code = &self.key_columns_code;
            let key_params_code = &self.key_params_code;

            let select = quote! {

            //#[derive(Debug, Eq, PartialEq, Hash)]
              // #vis struct #struct_key_ident ( #key_type_code);

                /* impl toql::key::Key for #struct_ident {
                    type Key = #struct_key_ident;

                    fn get_key(&self) -> toql::error::Result<Self::Key> {
                       Ok(  #struct_key_ident (#key_getter) )
                    }
                    fn set_key(&mut self, key: Self::Key) -> toql::error::Result<()> {
                      #( #key_setters)*
                      Ok(())
                    }
                    fn columns() ->Vec<String> {
                         let mut columns: Vec<String>= Vec::new();

                        #(#key_columns_code)*
                        columns
                    }
                    fn params(key: &Self::Key) ->Vec<String> {
                         let mut params: Vec<String>= Vec::new();

                        #(#key_params_code)*
                        params
                    }
                }

                impl std::convert::TryFrom<#struct_ident> for #struct_key_ident
                {
                    type Error = toql::error::ToqlError;
                    fn try_from(entity: #struct_ident) -> toql::error::Result<Self> {
                        <#struct_ident as toql::key::Key>::get_key(&entity)
                    }
                }

                // Impl to supprt HashSets
                impl std::hash::Hash for #struct_ident {
                    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        <#struct_ident as toql::key::Key>::get_key(self).ok().hash(state);
                    }
                } */

               /*  #vis fn #key_predicate_fn (key: #struct_key_ident) ->Result<toql::query::Query , toql::error::ToqlError>{
                    Ok(toql::query::Query::new() #(#key_predicates)* )
                } */

                /* impl toql::query_builder::KeyPredicate for #struct_key_ident {

                    fn key_predicate(&self) -> Result<toql::query::Query , toql::error::ToqlError>
                        {
                            let key = self;
                        Ok(toql::query::Query::new() #(#key_predicates)* )
                    }
                } */

                impl<'a> toql::mysql::select::Select<#struct_ident> for #struct_ident {


                    fn columns_sql() -> String {
                           #columns_sql_code

                    }
                    fn joins_sql() -> String {
                            #joins_sql_code
                    }
                    fn select_sql(join: Option<&str>) -> String {
                            format!( #select_statement,
                            Self::columns_sql(), Self::joins_sql(),join.unwrap_or(""))
                    }


                     fn select_one<C>(key: &<#struct_ident as toql::key::Key>::Key, conn: &mut C)
                     -> Result<#struct_ident,  toql::error::ToqlError>
                     where C: toql::mysql::mysql::prelude::GenericConnection
                     {
                        let select_stmt = format!( "{} WHERE {} LIMIT 0,2", Self::select_sql(None), vec![ #(#select_keys),*].join( " AND "));

                        let mut params :Vec<String> = Vec::new();

                        #(#select_keys_params)*
                        toql::log_sql!(select_stmt, params);

                        let entities_stmt = conn.prep_exec(select_stmt, &params)?;
                        let mut entities = toql::mysql::row::from_query_result::< #struct_ident>(entities_stmt)?;

                        if entities.len() > 1 {
                            return Err( toql::error::ToqlError::NotUnique);
                        } else if entities.is_empty() {
                            return Err( toql::error::ToqlError::NotFound);
                        }

                        let key_predicate = vec![ #(#select_keys),*].join( " AND ");
                        #(#merge_code)*
                        Ok(entities.pop().unwrap())
                     }


                        fn select_many<C>(
                            key: &<#struct_ident as toql::key::Key>::Key,
                            conn: &mut C,
                            first: u64,
                            max: u16
                        ) -> Result<Vec< #struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                        {
                                unimplemented!();


                        }

                        fn select_dependencies<C>(join: &str, params:&Vec<String>,
                            conn: &mut C) -> Result<Vec<#struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                            {
                                let select_stmt =  Self::select_sql(Some(join));
                                //let select_stmt = format!( #select_dependend_stmt, Self::columns_sql(), Self::joins_sql(), join);




                        toql::log_sql!(select_stmt, params);

                        let entities_stmt = conn.prep_exec(select_stmt, params)?;
                        let mut entities = toql::mysql::row::from_query_result::<#struct_ident>(entities_stmt)?;


                        let key_predicate = vec![ #(#select_keys),*].join( " AND ");
                        #(#merge_code)*

                        Ok(entities)
                        }



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
