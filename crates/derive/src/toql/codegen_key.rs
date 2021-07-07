use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct CodegenKey<'a> {
    rust_struct: &'a crate::sane::Struct,
    key_fields_code: Vec<TokenStream>,
    key_columns_code: Vec<TokenStream>,
    key_inverse_columns_code: Vec<TokenStream>,
    key_params_code: Vec<TokenStream>,
    key_types: Vec<TokenStream>,
    key_fields: Vec<TokenStream>,
    key_setters: Vec<TokenStream>,
    key_getters: Vec<TokenStream>,
    // key_sql_predicates: Vec<TokenStream>,
    /*  partial_key_types: Vec<TokenStream>,
    partial_key_sql_predicates: Vec<TokenStream>, */
    key_field_declarations: Vec<TokenStream>,

    toql_eq_predicates: Vec<TokenStream>,
    toql_eq_foreign_predicates: Vec<TokenStream>,

    key_constr_code: Vec<TokenStream>,

    sql_arg_code: Option<TokenStream>,
    //_code : Option<TokenStream>,
    try_from_setters: Vec<TokenStream>,
}

impl<'a> CodegenKey<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenKey {
        CodegenKey {
            rust_struct: &toql,
            key_fields_code: Vec::new(),
            key_columns_code: Vec::new(),
            key_inverse_columns_code: Vec::new(),
            key_params_code: Vec::new(),
            key_types: Vec::new(),
            key_fields: Vec::new(),
            key_setters: Vec::new(),
            key_getters: Vec::new(),

            key_field_declarations: Vec::new(),
            //  key_sql_predicates: Vec::new(),
            /*    partial_key_types: Vec::new(),
            partial_key_sql_predicates: Vec::new(), */
            toql_eq_predicates: Vec::new(),
            toql_eq_foreign_predicates: Vec::new(),

            key_constr_code: Vec::new(),
            sql_arg_code: None,
            // slice_to_query_code : None,
            try_from_setters: Vec::new(),
        }
    }

    pub fn add_key_field(&mut self, field: &crate::sane::Field) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_base_type_ident = &field.rust_base_type_ident;
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let toql_field_name = &field.toql_field_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    return Ok(());
                }

                if let SqlTarget::Column(ref column) = &regular_attrs.sql_target {
                    self.key_columns_code
                        .push(quote!( columns.push( String::from(#column)); ));

                    self.key_fields_code
                        .push(quote!( fields.push( String::from(#toql_field_name)); ));

                    if let Some(inverse_column) = &regular_attrs.default_inverse_column {
                        self.key_inverse_columns_code
                            .push(quote!( columns.push( String::from(#inverse_column)); ));
                    }

                    self.toql_eq_predicates.push( quote!(.and(toql::query::field::Field::from(#toql_field_name).eq(&t. #rust_field_ident))));
                    self.toql_eq_foreign_predicates.push( quote!(.and( toql::query::field::Field::from(format!("{}_{}",toql_path ,#toql_field_name)).eq(&t.#rust_field_ident))));

                    self.key_constr_code.push(quote!(#rust_field_ident));

                    self.key_field_declarations
                        .push(quote!( pub #rust_field_ident: #rust_type_ident));

                    self.sql_arg_code = if self.key_columns_code.len() == 1 {
                        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;
                        let struct_key_ident =
                            Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
                        Some(quote!(
                            impl From< #struct_key_ident> for toql::sql_arg::SqlArg {
                                fn from( t: #struct_key_ident) -> toql::sql_arg::SqlArg {
                                    toql::sql_arg::SqlArg::from(t. #rust_field_ident)
                                }
                            }
                            impl From<&#struct_key_ident> for toql::sql_arg::SqlArg {
                                fn from(t: &#struct_key_ident) -> toql::sql_arg::SqlArg {
                                    toql::sql_arg::SqlArg::from(t. #rust_field_ident .to_owned())
                                }
                            }
                        ))
                    } else {
                        None
                    };

                    
                } else {
                    return Err(darling::Error::custom(
                        "Key must not be an expression.".to_string(),
                    )
                    .with_span(&field.rust_field_ident));
                }

                //self.key_types.push(quote!( #rust_type_ident));
                self.key_types.push(quote!( #rust_base_type_ident)); 

                if field.number_of_options > 0 {
                    return Err(
                        darling::Error::custom("Key must not be optional.".to_string())
                            .with_span(&field.rust_field_ident),
                    );
                    /*  let value = quote!(self. #rust_field_ident .as_ref()
                                  .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))?
                                  .to_owned());
                      self.key_fields.push(value);
                      self.key_getters.push(quote!(#rust_field_ident : self. #rust_field_ident .as_ref()
                                  .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))?
                                  .to_owned()));

                    //  let index = syn::Index::from(self.key_types.len() - 1);
                      self.key_setters
                          .push(quote!(self. #rust_field_ident = Some( key . #rust_field_ident  ) )); */
                } else {
                    self.key_fields
                        .push(quote!(self. #rust_field_ident .to_owned()));

                    self.key_getters
                        .push(quote!(#rust_field_ident : self. #rust_field_ident .to_owned()));

                    // let index = syn::Index::from(self.key_types.len() - 1);

                    self.key_setters
                        .push(quote!(self. #rust_field_ident = key . #rust_field_ident));
                }
                let try_from_setters_index = syn::Index::from(self.try_from_setters.len());
                self.try_from_setters
                    .push(quote!( #rust_field_ident : args
                                .get(#try_from_setters_index)
                                .ok_or(toql::error::ToqlError::ValueMissing( #rust_field_name.to_string()))?
                                .try_into()?)); // Better Error

                // let key_index = syn::Index::from(self.key_fields.len() - 1);

                self.key_params_code.push(
                    quote!(params.push(toql::sql_arg::SqlArg::from(&key . #rust_field_ident)); ),
                );
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }
                let default_self_column_code = &join_attrs.default_self_column_code;

               
                self.key_types
                    .push(quote!( <#rust_base_type_ident as toql::keyed::Keyed>::Key));

                let toql_name = &field.toql_field_name;
                self.toql_eq_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #rust_field_ident, #toql_name))));
                self.toql_eq_foreign_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #rust_field_ident, #toql_name))));

                self.key_constr_code.push(quote!(#rust_field_ident));

                self.key_field_declarations.push(
                    quote!( pub #rust_field_ident: <#rust_type_ident as toql::keyed::Keyed>::Key),
                );

               
                let columns_map_code = &join_attrs.columns_map_code;
                self.key_columns_code.push(quote!(

                <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|other_column| {
                    #default_self_column_code;
                    let column = #columns_map_code;
                    columns.push(column.to_string());
                });
                ));

                self.key_fields_code.push(quote!(
                 <<#rust_type_ident as toql::keyed::Keyed>::Key as toql :: key_fields :: KeyFields> :: fields().iter().for_each(|other_field| {
                    fields.push(format!("{}_{}",#toql_field_name, other_field));
                });
                ));

                self.key_inverse_columns_code.push( quote!(

                        <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns().iter().for_each(|other_column| {
                            #default_self_column_code;
                            let column = #columns_map_code;
                            columns.push(column.to_string());
                        });
                        ));

                //self.key_params_code.push( quote!( params.extend_from_slice(&<#rust_type_ident as toql::key::Key>::params(& key. #key_index));));
                self.key_params_code.push( quote!( params.extend_from_slice(&toql::key::Key::params(& key. #rust_field_ident)); ));

                // Select key predicate
                if field.number_of_options > 0 {
                    // Raise error key must not be optional
                    return Err(
                        darling::Error::custom("Key must not be optional.".to_string())
                            .with_span(&field.rust_field_ident),
                    );
                  
                } else {
                    self.key_fields.push(quote!(
                        < #rust_type_ident as toql::keyed::Keyed>::key(  &self. #rust_field_ident )
                    ));
                    self.key_getters.push(quote!(#rust_field_ident :
                        < #rust_type_ident as toql::keyed::Keyed>::key(  &self. #rust_field_ident )
                    ));

                    self.key_setters.push( quote!(
                                    < #rust_type_ident as toql::keyed::KeyedMut>::set_key(&mut self. #rust_field_ident,key . #rust_field_ident)
                            ));
                }

                let try_from_setters_index = syn::Index::from(self.try_from_setters.len());
                self.try_from_setters
                .push(quote!( #rust_field_ident :
                                        <#rust_type_ident as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[ #try_from_setters_index..]))?
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

impl<'a> quote::ToTokens for CodegenKey<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.rust_struct.rust_struct_visibility;
        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;

        let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
        let key_fields_code = &self.key_fields_code;
        let key_columns_code = &self.key_columns_code;
        let key_inverse_columns_code = &self.key_inverse_columns_code;
        let key_params_code = &self.key_params_code;

        // let partial_key_types = &self.partial_key_types;
        let key_types = &self.key_types;

        //let key_getter = quote!( #(#key_fields  ),* );
        let key_getters = &self.key_getters;
        let key_setters = &self.key_setters;

        // Single type or tuple
        let key_type_arg = if self.key_types.len() == 1 {
            quote!( #(#key_types),* )
        } else {
            quote!( ( #( #key_types),*) )
        };
       
        let key_field_declarations = &self.key_field_declarations;
        
        let serde = if cfg!(feature = "serde") {
            quote!(toql::serde::Serialize, toql::serde::Deserialize,)
        } else {
            quote!()
        };

        let key_constr_code = if self.key_constr_code.len() == 1 {
            let key_constr_code = self.key_constr_code.get(0).unwrap();
            vec![quote!( #key_constr_code : key)]
        } else {
            self.key_constr_code
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let index = syn::Index::from(i);
                    quote!(#k: key. #index)
                })
                .collect::<Vec<_>>()
        };

        let sql_arg_code = &self.sql_arg_code;

        let try_from_setters = &self.try_from_setters;

        let key = quote! {


                #[derive(Debug, Eq, PartialEq, Hash, #serde Clone)]

                   #vis struct #struct_key_ident {
                       #(#key_field_declarations),*
                    }

                     impl toql::key_fields::KeyFields  for #struct_key_ident {
                            type Entity = #rust_stuct_ident;

                            fn fields() ->Vec<String> {
                             let mut fields: Vec<String>= Vec::new();

                                #(#key_fields_code)*
                                fields
                            }

                            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                                let mut params: Vec<toql::sql_arg::SqlArg>= Vec::new();
                                let key = self; // TODO cleanup

                                #(#key_params_code)*
                                params
                            }
                       }
                        impl toql::key_fields::KeyFields  for &#struct_key_ident {
                            type Entity = #rust_stuct_ident;

                            fn fields() ->Vec<String> {
                                <#struct_key_ident as toql::key_fields::KeyFields>::fields()
                            }

                            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                                <#struct_key_ident as toql::key_fields::KeyFields>::params(self)
                            }
                       }

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
                            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                                let mut params: Vec<toql::sql_arg::SqlArg>= Vec::new();
                                let key = self; // TODO cleanup

                                #(#key_params_code)*
                                params
                            }
                       }

                        impl toql::key::Key for &#struct_key_ident {
                            type Entity = #rust_stuct_ident;
                            fn columns() -> Vec<String> {
                                <#struct_key_ident as toql::key::Key>::columns()
                            }
                            fn default_inverse_columns() -> Vec<String> {
                            <#struct_key_ident as toql::key::Key>::default_inverse_columns()
                            }
                            fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
                                <#struct_key_ident as toql::key::Key>::params(self)
                            }
                        }
                     

                       #sql_arg_code


                   


                    impl toql::keyed::Keyed for #rust_stuct_ident {
                        type Key = #struct_key_ident;

                        fn key(&self) -> Self::Key {
                            #struct_key_ident {
                               #( #key_getters),*
                               }
                        }
                    }

                     impl toql::keyed::Keyed for &#rust_stuct_ident {
                        type Key = #struct_key_ident;

                        fn key(&self) -> Self::Key {
                            <#rust_stuct_ident as toql::keyed::Keyed>::key(self)
                        }
                    }
                    impl toql::keyed::Keyed for &mut #rust_stuct_ident {
                        type Key = #struct_key_ident;

                        fn key(&self) -> Self::Key {
                             <#rust_stuct_ident as toql::keyed::Keyed>::key(self)
                        }
                    }

                 

                    impl toql::keyed::KeyedMut for #rust_stuct_ident {

                        fn set_key(&mut self, key: Self::Key)  {
                          #( #key_setters;)*
                        }

                    }

                     impl toql::keyed::KeyedMut for &mut #rust_stuct_ident {

                        fn set_key(&mut self, key: Self::Key)  {
                            <#rust_stuct_ident as toql::keyed::KeyedMut>::set_key(self, key)
                        }

                    }
       
                    impl std::convert::TryFrom<Vec< toql::sql_arg::SqlArg>> for #struct_key_ident
                    {
                        type Error = toql::error::ToqlError;
                        fn try_from(args: Vec< toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
                            use std::convert::TryInto;

                           Ok(#struct_key_ident {
                               #( #try_from_setters),*
                           })

                        }
                    }

                    impl std::convert::From<#key_type_arg> for #struct_key_ident
                    {

                        fn from(key: #key_type_arg) ->Self {
                            Self{
                                #(#key_constr_code),*
                            }
                        }
                    }


                    // Impl to support HashSets
                    impl std::hash::Hash for #rust_stuct_ident {
                        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                            <#rust_stuct_ident as toql::keyed::Keyed>::key(self).hash(state);
                        }
                    }

                    //#partial_key

                };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
