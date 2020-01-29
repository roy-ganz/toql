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
    serde_key: bool,

    forward_key_columns : usize,
    forward_key_joins: Vec<TokenStream>,
    mysql_deserialize_key: Vec<TokenStream>,
    mysql_forward_join_key: Vec<TokenStream>
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
            serde_key: toql.serde_key,
            forward_key_columns : 0,
            forward_key_joins: Vec::new(),
            mysql_deserialize_key: Vec::new(),
            mysql_forward_join_key: Vec::new(),
        }
    }

    pub fn add_key_field(&mut self, field: &crate::sane::Field) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_type_name = &field.rust_type_name;
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;

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
                } else {
                    // TODO Raise error
                }

                self.key_types.push(quote!( #rust_type_ident));

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
                
                let error_field = format!("{}Key::{}", &self.rust_struct.rust_struct_ident, rust_field_name);
                 let increment = if self.mysql_deserialize_key.is_empty() {
                    quote!(*i)
                } else {
                    quote!({*i = *i + 1;*i})
                };
                self.mysql_deserialize_key.push(quote!(
                    row.take_opt( #increment).unwrap()
                                .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string())
                            )?
                ));
                self.forward_key_columns = self.forward_key_columns + 1;

                let key_index = syn::Index::from(self.key_fields.len() - 1);

                self.key_params_code
                    .push(quote!(params.push(key . #key_index .to_owned().to_string()); ));
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }
                let default_self_column_code = &join_attrs.default_self_column_code;
                let key_index = syn::Index::from(self.key_types.len());
                self.key_types
                    .push(quote!( <#rust_type_ident as toql::key::Key>::Key));
                let columns_map_code= &join_attrs.columns_map_code;
                self.key_columns_code.push( quote!( 
                    
                        <#rust_type_ident as toql::key::Key>::columns().iter().for_each(|other_column| {
                            #default_self_column_code;
                            let column = #columns_map_code;
                            columns.push(column.to_string());
                        });
                        ));

                        self.key_inverse_columns_code.push( quote!( 
                    
                        <#rust_type_ident as toql::key::Key>::default_inverse_columns().iter().for_each(|other_column| {
                            #default_self_column_code;
                            let column = #columns_map_code;
                            columns.push(column.to_string());
                        });
                        ));
                

                self.key_params_code.push( quote!( params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params(& key. #key_index));));

                // Select key predicate
                if field.number_of_options > 0 {
                    self.key_fields.push( quote!(
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
                    self.key_fields.push(quote!(
                        < #rust_type_ident as toql::key::Key>::get_key(  &self. #rust_field_ident )?
                    ));

                    self.key_setters.push( quote!(
                                    < #rust_type_ident as toql::key::Key>::set_key(&mut self. #rust_field_ident,key . #key_index)?;
                            ));
                }

                // Impl key from result row
                self.mysql_forward_join_key.push(quote!(
                   *i = < #rust_type_ident > ::forward_row(*i);
                ));
                 let increment = if self.mysql_deserialize_key.is_empty() {
                    quote!(i)
                } else {
                    quote!({*i = *i + 1; i})
                };
                self.mysql_deserialize_key.push(quote!(
                     << #rust_type_ident as toql :: key :: Key > :: Key >:: from_row_with_index (row, #increment)?
                ));
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

        let key_types = &self.key_types;

        let key_type_code = quote!(  #(pub #key_types),* );

        let key_fields = &self.key_fields;

        let key_getter = quote!( #(#key_fields  ),* );
        let key_setters = &self.key_setters;

        // Single type or tuple
        let key_type_arg =  if  self.key_types.len() == 1 {
            quote!( #(#key_types),* )
        } else {
            quote!( ( #( #key_types),*) )
        };
         let key_index_code =  if  self.key_types.len() == 1 {
            quote!( key )
        } else {
            let key_codes = key_types.iter().enumerate()
                .map(|(i, _)| {let index = syn::Index::from(i); quote!( key. #index) })
                .collect::<Vec<_>>(); 
            quote!(  #( #key_codes),* )
        };

        let serde = if self.serde_key {
            quote!( ,Deserialize, Serialize)
        } else { 
            quote!()
        };

        let struct_key_wrapper_ident = Ident::new(&format!("{}Keys", &rust_stuct_ident), Span::call_site());

        let forward_key_columns = &self.forward_key_columns;
        let forward_key_joins=  &self.forward_key_joins;
        let mysql_deserialize_key= &self.mysql_deserialize_key;

        let key = quote! {

        #[derive(Debug, Eq, PartialEq, Hash #serde, Clone)]
           #vis struct #struct_key_ident ( #key_type_code);

            impl toql::key::Key for #rust_stuct_ident {
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
                 fn default_inverse_columns() ->Vec<String> {
                     let mut columns: Vec<String>= Vec::new();

                    #(#key_inverse_columns_code)*
                    columns
                }
                fn params(key: &Self::Key) ->Vec<String> {
                     let mut params: Vec<String>= Vec::new();

                    #(#key_params_code)*
                    params
                }
            }

            impl std::convert::TryFrom<#rust_stuct_ident> for #struct_key_ident
            {
                type Error = toql::error::ToqlError;
                fn try_from(entity: #rust_stuct_ident) -> toql::error::Result<Self> {
                    <#rust_stuct_ident as toql::key::Key>::get_key(&entity)
                }
            }

            impl std::convert::From<#key_type_arg> for #struct_key_ident
            {
                
                fn from(key: #key_type_arg) ->Self {
                    Self( #key_index_code )
                }
            }


            // Impl to supprt HashSets
            impl std::hash::Hash for #rust_stuct_ident {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    <#rust_stuct_ident as toql::key::Key>::get_key(self).ok().hash(state);
                }
            }


        /*     impl Into<toql::query::Query> for #struct_key_ident {
                    fn into(self) -> toql::query::Query {
                            #struct_key_wrapper_ident( vec![self]).into()
                    }
                }

            impl Into<toql::query::Query> for &#struct_key_ident {
                fn into(self) -> toql::query::Query {
                            self.to_owned().into()
                }
            } */

           
            impl toql :: mysql :: row:: FromResultRow < #struct_key_ident > for #struct_key_ident {
           
            fn forward_row(mut i : usize) -> usize {
                i = i + #forward_key_columns;
                #(#forward_key_joins)*
                i
            }

            fn from_row_with_index ( mut row : & mut toql::mysql::mysql :: Row , i : &mut usize) 
                -> toql :: mysql :: error:: Result < #struct_key_ident> {

                Ok ( #struct_key_ident(
                    #(#mysql_deserialize_key),*

                ))
            }
            }
            
                 
                #[derive(Debug, Eq, PartialEq, Hash, Clone #serde)]
                pub struct #struct_key_wrapper_ident(pub Vec<#struct_key_ident>);

                /* impl<T: IntoIterator<Item = #struct_key_ident>> std::ops::Deref for #struct_key_wrapper_ident<T> {
                    type Target = T;

                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }


 */
                impl Into<#struct_key_wrapper_ident> for #struct_key_ident  {
                    fn into(self) -> #struct_key_wrapper_ident {
                          #struct_key_wrapper_ident(vec![self])
                    }
                }

                impl IntoIterator for #struct_key_wrapper_ident {
                    type Item = #struct_key_ident;
                    type IntoIter = ::std::vec::IntoIter<Self::Item>;

                    fn into_iter(self) -> Self::IntoIter {
                        self.0.into_iter()
                    }
                }
        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
