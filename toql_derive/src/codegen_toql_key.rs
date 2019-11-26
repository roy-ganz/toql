use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct GeneratedToqlKey<'a> {
    rust_struct: &'a crate::sane::Struct,
    key_columns_code: Vec<TokenStream>,
    key_params_code: Vec<TokenStream>,
    key_types: Vec<TokenStream>,
    key_fields: Vec<TokenStream>,
    key_setters: Vec<TokenStream>,
    serde_key: bool
}

impl<'a> GeneratedToqlKey<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlKey {
        GeneratedToqlKey {
            rust_struct: &toql,
            key_columns_code: Vec::new(),
            key_params_code: Vec::new(),
            key_types: Vec::new(),
            key_fields: Vec::new(),
            key_setters: Vec::new(),
            serde_key: toql.serde_key
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

                let key_index = syn::Index::from(self.key_fields.len() - 1);

                self.key_params_code
                    .push(quote!(params.push(key . #key_index .to_owned().to_string()); ));
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }

                let key_index = syn::Index::from(self.key_types.len());
                self.key_types
                    .push(quote!( <#rust_type_ident as toql::key::Key>::Key));

                self.key_columns_code.push( quote!( columns.extend_from_slice(&<#rust_type_ident as toql::key::Key>::columns());));
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
        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
