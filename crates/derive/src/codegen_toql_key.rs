use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct GeneratedToqlKey<'a> {
    rust_struct: &'a crate::sane::Struct,
    key_columns_code: Vec<TokenStream>,
    key_inverse_columns_code: Vec<TokenStream>,
    key_params_code: Vec<TokenStream>,
    key_types: Vec<TokenStream>,
    key_fields: Vec<TokenStream>,
    key_setters: Vec<TokenStream>,
    key_sql_predicates: Vec<TokenStream>,
    partial_key_types: Vec<TokenStream>,
    partial_key_sql_predicates: Vec<TokenStream>,
    serde_key: bool,
}

impl<'a> GeneratedToqlKey<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlKey {
        GeneratedToqlKey {
            rust_struct: &toql,
            key_columns_code: Vec::new(),
            key_inverse_columns_code: Vec::new(),
            key_params_code: Vec::new(),
            key_types: Vec::new(),
            key_fields: Vec::new(),
            key_setters: Vec::new(),
            key_sql_predicates: Vec::new(),
            partial_key_types: Vec::new(),
            partial_key_sql_predicates: Vec::new(),
            serde_key: toql.serde_key,
        }
    }

    pub fn add_key_field(&mut self, field: &crate::sane::Field) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_type_name = &field.rust_type_name;
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let key_index = syn::Index::from(self.key_fields.len());

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    return Ok(());
                }
                
                if let SqlTarget::Column(ref column) = &regular_attrs.sql_target {
                    self.key_columns_code
                        .push(quote!( columns.push( String::from(#column)); ));

                    if let Some(inverse_column) = &regular_attrs.default_inverse_column {
                        self.key_inverse_columns_code
                            .push(quote!( columns.push( String::from(#inverse_column)); ));
                    }

                    // sql predicate trait 
                    let column_format = format!("{{}}.{} = ? AND ", column);
                    self.partial_key_sql_predicates.push( quote!(
                        if let Some(v) = &self.#rust_field_ident {
                            predicate.push_str( &format!(#column_format, alias));
                            params.push(toql::sql::SqlArg::from(v));
                        }
                    ));
                     self.key_sql_predicates.push( quote!(
                            predicate.push_str( &format!(#column_format, alias));
                            params.push( toql::sql::SqlArg::from(&self. #key_index));
                    ));

                } else {
                    // TODO Raise error
                }

                self.key_types.push(quote!( #rust_type_ident));
                self.partial_key_types.push(quote!(#rust_field_ident : Option<#rust_type_ident>));

                

                if field.number_of_options > 0 {
                    let value = quote!(self. #rust_field_ident .as_ref() .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))? .to_owned());
                    self.key_fields.push(value);

                    let index = syn::Index::from(self.key_types.len() - 1);
                    self.key_setters
                        .push(quote!(self. #rust_field_ident = Some( key . #index  ); ))
                } else {
                    self.key_fields
                        .push(quote!(self. #rust_field_ident .to_owned()));

                    let index = syn::Index::from(self.key_types.len() - 1);

                    self.key_setters
                        .push(quote!(self. #rust_field_ident = key . #index;))
                }

                let key_index = syn::Index::from(self.key_fields.len() - 1);

                self.key_params_code
                    .push(quote!(params.push(toql::sql::SqlArg::from(&key . #key_index)); ));
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }
                let default_self_column_code = &join_attrs.default_self_column_code;
               // let key_index = syn::Index::from(self.key_types.len());
                self.key_types.push(quote!( <#rust_type_ident as toql::key::Keyed>::Key));
                self.partial_key_types.push(quote!(#rust_field_ident : Option<<#rust_type_ident as toql::key::Keyed>::Key>));

                // sql Prediate trait
                 let join_alias = &join_attrs.join_alias;
                self.partial_key_sql_predicates.push( quote!(
                    if let Some(v) = &self.#rust_field_ident {
                    let (pr, pa) = <<#rust_type_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_predicate(&v, #join_alias);
                    predicate.push_str( &pr);
                    predicate.push_str(" AND ");
                    params.extend_from_slice(&pa);
                } 
                ));
                  self.key_sql_predicates.push( quote!(
                    let (pr, pa) = <<#rust_type_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_predicate(&self. #key_index, #join_alias);
                    predicate.push_str( &pr);
                    params.extend_from_slice(&pa);
                ));


                let columns_map_code = &join_attrs.columns_map_code;
                self.key_columns_code.push(quote!(

                <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|other_column| {
                    #default_self_column_code;
                    let column = #columns_map_code;
                    columns.push(column.to_string());
                });
                ));

                self.key_inverse_columns_code.push( quote!(

                        <<#rust_type_ident as toql::key::Keyed>::Key as toql::key::Key>::default_inverse_columns().iter().for_each(|other_column| {
                            #default_self_column_code;
                            let column = #columns_map_code;
                            columns.push(column.to_string());
                        });
                        ));

                //self.key_params_code.push( quote!( params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params(& key. #key_index));));
                self.key_params_code.push( quote!( params.extend_from_slice(&toql::key::Key::params(& key. #key_index)); ));

                // Select key predicate
                if field.number_of_options > 0 {
                    self.key_fields.push( quote!(
                                < #rust_type_ident as toql::key::Keyed>::try_get_key(
                                    self. #rust_field_ident .as_ref()
                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))?
                                    )?
                            ));

                    self.key_setters.push( quote!(
                                        < #rust_type_ident as toql::key::Keyed>::try_set_key(self. #rust_field_ident .as_mut()
                                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))? , key . #key_index )?;
                            ));
                } else {
                    self.key_fields.push(quote!(
                        < #rust_type_ident as toql::key::Keyed>::try_get_key(  &self. #rust_field_ident )?
                    ));

                    self.key_setters.push( quote!(
                                    < #rust_type_ident as toql::key::Keyed>::try_set_key(&mut self. #rust_field_ident,key . #key_index)?;
                            ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn key_missing(&self) -> bool {
        self.key_types.is_empty()
    }
}

impl<'a> quote::ToTokens for GeneratedToqlKey<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.rust_struct.rust_struct_visibility;
        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;

        let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
        let key_columns_code = &self.key_columns_code;
        let key_inverse_columns_code = &self.key_inverse_columns_code;
        let key_params_code = &self.key_params_code;

        let partial_key_types = &self.partial_key_types;
        let key_types = &self.key_types;

        let key_type_code = quote!(  #(pub #key_types),* );

        let key_fields = &self.key_fields;

        let key_getter = quote!( #(#key_fields  ),* );
        let key_setters = &self.key_setters;

        // Single type or tuple
        let key_type_arg = if self.key_types.len() == 1 {
            quote!( #(#key_types),* )
        } else {
            quote!( ( #( #key_types),*) )
        };
        let key_index_code = if self.key_types.len() == 1 {
            quote!(key)
        } else {
            let key_codes = key_types
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let index = syn::Index::from(i);
                    quote!( key. #index)
                })
                .collect::<Vec<_>>();
            quote!(  #( #key_codes),* )
        };

        let serde = if self.serde_key {
            quote!( ,Deserialize, Serialize)
        } else {
            quote!()
        };
        let key_sql_predicates = &self.key_sql_predicates;

        let partial_key = if key_types.len() > 1 {
            let struct_key_ident = Ident::new(&format!("{}PartialKey", &rust_stuct_ident), Span::call_site());
            let partial_key_sql_predicates = &self.partial_key_sql_predicates;
            quote!(
                #[derive(Debug, Eq, PartialEq, Hash #serde, Clone)]
                #vis struct #struct_key_ident { #(pub #partial_key_types),* }


               

                impl toql::sql_predicate::SqlPredicate  for #struct_key_ident {

                    type Entity = #rust_stuct_ident;

                fn sql_predicate(&self, alias: &str) ->  toql::sql::Sql {
                    let mut predicate = String::new();
                    let mut params: Vec<toql::sql::SqlArg> = Vec::new();

                    #(#partial_key_sql_predicates)*

                     if !predicate.is_empty() {
                        predicate.pop();
                        predicate.pop();
                        predicate.pop();
                        predicate.pop();
                    }
                   
                    (predicate, params)
                }
            }
            )

        } else {
            quote!()
        };

        let key = quote! {

        #[derive(Debug, Eq, PartialEq, Hash #serde, Clone)]
           #vis struct #struct_key_ident ( #key_type_code);

            impl toql::key::Key  for #struct_key_ident {
                    type Entity = #rust_stuct_ident;

                    fn columns() ->Vec<String> {
                     let mut columns: Vec<String>= Vec::new();

                        #(#key_columns_code)*
                        columns
                    }
                    fn default_inverse_columns() ->Vec<String> {
                        let mut columns: Vec<String>= Vec::new();

                        #(#key_inverse_columns_code)*
                        columns
                    }
                    fn params(&self) ->Vec<toql::sql::SqlArg> {
                        let mut params: Vec<toql::sql::SqlArg>= Vec::new();
                        let key = self; // TODO cleanup

                        #(#key_params_code)*
                        params
                    }

                   /*  fn sql_predicate(&seld, alias:&str) -> (String, Vec<String>) {
                        let predicate = String::new();
                        let params: Vec<String> = Vec::new();

                        #(#partial_key_sql_predicates)*

                        if !predicate.is_empty() {
                            predicate.pop();
                            predicate.pop();
                            predicate.pop();
                            predicate.pop();
                        }
                    
                        (predicate, params)

                    } */
                }

             impl toql::sql_predicate::SqlPredicate  for #struct_key_ident {
                 type Entity = #rust_stuct_ident;

                fn sql_predicate(&self, alias: &str) -> toql::sql::Sql {
                    let mut predicate = String::new();
                    let mut params: Vec<toql::sql::SqlArg> = Vec::new();

                    #(#key_sql_predicates)*

                     if !predicate.is_empty() {
                        predicate.pop();
                        predicate.pop();
                        predicate.pop();
                        predicate.pop();
                    }
                   
                    (predicate, params)
                }
             }

           

            impl toql::key::Keyed for #rust_stuct_ident {
                type Key = #struct_key_ident;

                fn try_get_key(&self) -> toql::error::Result<Self::Key> {
                   Ok(  #struct_key_ident (#key_getter) )
                }
                fn try_set_key(&mut self, key: Self::Key) -> toql::error::Result<()> {
                  #( #key_setters)*
                  Ok(())
                }
                
            }

            impl std::convert::TryFrom<#rust_stuct_ident> for #struct_key_ident
            {
                type Error = toql::error::ToqlError;
                fn try_from(entity: #rust_stuct_ident) -> toql::error::Result<Self> {
                    <#rust_stuct_ident as toql::key::Keyed>::try_get_key(&entity)
                }
            }

            impl std::convert::From<#key_type_arg> for #struct_key_ident
            {

                fn from(key: #key_type_arg) ->Self {
                    Self( #key_index_code )
                }
            }


            // Impl to support HashSets
            impl std::hash::Hash for #rust_stuct_ident {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    <#rust_stuct_ident as toql::key::Keyed>::try_get_key(self).ok().hash(state);
                }
            }

            #partial_key

        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}