use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct CodegenKey<'a> {
    rust_struct: &'a crate::sane::Struct,
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
    

    key_constr_code:Vec<TokenStream>,


    sql_arg_code : Option<TokenStream>,
    slice_to_query_code : Option<TokenStream>,
    
    try_from_setters:Vec<TokenStream>,
    
}

impl<'a> CodegenKey<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenKey {
        CodegenKey {
            rust_struct: &toql,
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
            sql_arg_code : None,
            slice_to_query_code : None,
            
            try_from_setters: Vec::new(),
          
        }
    }

    pub fn add_key_field(&mut self, field: &crate::sane::Field) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_type_name = &field.rust_type_name;
        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
      //  let key_index = syn::Index::from(self.key_fields.len());
        let toql_field_name = &field.toql_field_name;
        
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

                    self.toql_eq_predicates.push( quote!(.and(toql::query::field::Field::from(#toql_field_name).eq(&t. #rust_field_ident))));
                    self.toql_eq_foreign_predicates.push( quote!(.and( toql::query::field::Field::from(format!("{}_{}",toql_path ,#toql_field_name)).eq(&t.#rust_field_ident))));

                    self.key_constr_code.push(quote!(#rust_field_ident));
                    
                    self.key_field_declarations.push(quote!( pub #rust_field_ident: #rust_type_ident));

                    
                        self.sql_arg_code = if self.key_columns_code.len() == 1 { 
                             let rust_stuct_ident = &self.rust_struct.rust_struct_ident;
                             let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
                            Some(quote!(
                                impl Into<toql::sql_arg::SqlArg> for #struct_key_ident {
                                    fn into(self) -> toql::sql_arg::SqlArg {
                                        toql::sql_arg::SqlArg::from(self. #rust_field_ident)
                                    }
                                }
                                impl Into<toql::sql_arg::SqlArg> for &#struct_key_ident {
                                    fn into(self) -> toql::sql_arg::SqlArg {
                                        toql::sql_arg::SqlArg::from(self. #rust_field_ident .to_owned())
                                    }
                                }
                            ))

                        } else { None };

                        self.slice_to_query_code = if self.key_columns_code.len() == 1 { 
                             let rust_stuct_ident = &self.rust_struct.rust_struct_ident;
                            Some(quote!(
                                fn slice_to_query(entities: &[Self]) -> toql::query::Query<#rust_stuct_ident>
                                where Self: Sized,
                                {
                                    toql::query::Query::<#rust_stuct_ident>::new()
                                    .and(toql::query::field::Field::from(#toql_field_name).ins(entities))
                                }
                              
                            ))

                        } else { None };

                   

                } else {
                    // TODO Raise error
                }

                self.key_types.push(quote!( #rust_type_ident));
              //  self.partial_key_types.push(quote!(#rust_field_ident : Option<#rust_type_ident>));

                

                if field.number_of_options > 0 {
                    let value = quote!(self. #rust_field_ident .as_ref() 
                                .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))? 
                                .to_owned());
                    self.key_fields.push(value);
                    self.key_getters.push(quote!(#rust_field_ident : self. #rust_field_ident .as_ref() 
                                .ok_or(toql::error::ToqlError::ValueMissing( String::from(# rust_type_name)))? 
                                .to_owned()));

                  //  let index = syn::Index::from(self.key_types.len() - 1);
                    self.key_setters
                        .push(quote!(self. #rust_field_ident = Some( key . #rust_field_ident  ) ));
                } else {
                    self.key_fields
                        .push(quote!(self. #rust_field_ident .to_owned()));

                        self.key_getters.push(quote!(#rust_field_ident : self. #rust_field_ident .to_owned()));

                   // let index = syn::Index::from(self.key_types.len() - 1);

                    self.key_setters
                        .push(quote!(self. #rust_field_ident = key . #rust_field_ident));

                }
                let try_from_setters_index = syn::Index::from(self.try_from_setters.len());
                self.try_from_setters
                    .push(quote!( #rust_field_ident : args
                                .get(#try_from_setters_index)
                                .ok_or(toql::error::ToqlError::ValueMissing( #rust_field_name.to_string()))?
                                .try_into()?));   // Better Error 
                
               // let key_index = syn::Index::from(self.key_fields.len() - 1);

                self.key_params_code
                    .push(quote!(params.push(toql::sql_arg::SqlArg::from(&key . #rust_field_ident)); ));
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }
                let default_self_column_code = &join_attrs.default_self_column_code;
               // let key_index = syn::Index::from(self.key_types.len());
                self.key_types.push(quote!( <#rust_type_ident as toql::key::Keyed>::Key));

                let toql_name = &field.toql_field_name;
                self.toql_eq_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #rust_field_ident, #toql_name))));
                self.toql_eq_foreign_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #rust_field_ident, #toql_name))));
               
              
                self.key_constr_code.push(quote!(#rust_field_ident));
              
              self.key_field_declarations.push(quote!( pub #rust_field_ident: <#rust_type_ident as toql::key::Keyed>::Key));
 
              //  self.partial_key_types.push(quote!(#rust_field_ident : Option<<#rust_type_ident as toql::key::Keyed>::Key>));

                // sql Prediate trait
               //  let join_alias = &join_attrs.join_alias;
               /*  self.partial_key_sql_predicates.push( quote!(
                    if let Some(v) = &self.#rust_field_ident {
                    let (pr, pa) = <<#rust_type_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_predicate(&v, #join_alias);
                    predicate.push_str( &pr);
                    predicate.push_str(" AND ");
                    params.extend_from_slice(&pa);
                } 
                ));
                  self.key_sql_predicates.push( quote!(
                    let (pr, pa) = <<#rust_type_ident as toql::key::Keyed>::Key as toql::sql_predicate::SqlPredicate>::sql_predicate(&self. #rust_field_ident, #join_alias);
                    predicate.push_str( &pr);
                    params.extend_from_slice(&pa);
                ));
 */

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
                self.key_params_code.push( quote!( params.extend_from_slice(&toql::key::Key::params(& key. #rust_field_ident)); ));

                // Select key predicate
                if field.number_of_options > 0 {
                    self.key_fields.push( quote!(
                                < #rust_type_ident as toql::key::Keyed>::try_get_key(
                                    self. #rust_field_ident .as_ref()
                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))?
                                    )?
                            ));
                    self.key_getters.push( quote!(#rust_field_ident :
                                < #rust_type_ident as toql::key::Keyed>::try_get_key(
                                    self. #rust_field_ident .as_ref()
                                        .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))?
                                    )?
                            ));

                    self.key_setters.push( quote!(
                                        < #rust_type_ident as toql::key::Keyed>::try_set_key(self. #rust_field_ident .as_mut()
                                            .ok_or(toql::error::ToqlError::ValueMissing( String::from(#rust_field_name)))? , key . #rust_field_ident )?
                            ));
                } else {
                    self.key_fields.push(quote!(
                        < #rust_type_ident as toql::key::Keyed>::try_get_key(  &self. #rust_field_ident )?
                    ));
                    self.key_getters.push(quote!(#rust_field_ident :
                        < #rust_type_ident as toql::key::Keyed>::try_get_key(  &self. #rust_field_ident )?
                    ));

                    self.key_setters.push( quote!(
                                    < #rust_type_ident as toql::key::Keyed>::try_set_key(&mut self. #rust_field_ident,key . #rust_field_ident)?
                            ));
                }

                let try_from_setters_index = syn::Index::from(self.try_from_setters.len());
                self.try_from_setters
                .push(quote!( #rust_field_ident : 
                                        <#rust_type_ident as toql::key::Keyed>::Key::try_from(Vec::from(&args[ #try_from_setters_index..]))?
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
       /*  let key_index_code = if self.key_types.len() == 1 {
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
        }; */

       

        let key_field_declarations = &self.key_field_declarations;
        //let key_sql_predicates = &self.key_sql_predicates;

       /*  let partial_key = if key_types.len() > 1 {
            let struct_key_ident = Ident::new(&format!("{}PartialKey", &rust_stuct_ident), Span::call_site());
            let partial_key_sql_predicates = &self.partial_key_sql_predicates;
            quote!(
                #[derive(Debug, Eq, PartialEq, Hash #serde, Clone)]
                #vis struct #struct_key_ident { #(pub #partial_key_types),* }


               

                impl toql::sql_predicate::SqlPredicate  for #struct_key_ident {

                    type Entity = #rust_stuct_ident;

                fn sql_predicate(&self, alias: &str) ->  toql::sql::Sql {
                    let mut predicate = String::new();
                    let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();

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
        }; */
     let serde = if cfg!(feature = "serde") {
         quote!(Serialize, Deserialize, )
     }else { quote!()};
     

    let key_constr_code =  if self.key_constr_code.len() == 1 {
        let key_constr_code = self.key_constr_code.get(0).unwrap();
        vec![quote!( #key_constr_code : key)]
    } else {
        self.key_constr_code.iter().enumerate()
            .map(|(i, k)| { let index = syn::Index::from(i); quote!(#k: key. #index)})
            .collect::<Vec<_>>()
    };
    let toql_eq_predicates = &self.toql_eq_predicates;
    let toql_eq_foreign_predicates = &self.toql_eq_foreign_predicates;
    let slice_to_query_code = &self.slice_to_query_code;
    let sql_arg_code = &self.sql_arg_code;   

    let try_from_setters = &self.try_from_setters;   

        let key = quote! {

        
        #[derive(Debug, Eq, PartialEq, Hash, #serde Clone)]
        
           #vis struct #struct_key_ident { 
               #(#key_field_declarations),* 
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

                impl Into<toql::query::Query<#rust_stuct_ident>>  for #struct_key_ident {
                
                fn into(self) ->toql::query::Query<#rust_stuct_ident> {
                     <#struct_key_ident as toql::to_query::ToQuery<#rust_stuct_ident>>::to_query(&self)
                }
             }  
               impl toql::to_query::ToQuery<#rust_stuct_ident> for  #struct_key_ident {
                
                fn to_query(&self) ->toql::query::Query<#rust_stuct_ident> {
                    let t = self;
                    toql::query::Query::<#rust_stuct_ident>::new()
                    #(#toql_eq_predicates)*
                }
                #slice_to_query_code
               }

               #sql_arg_code

                
             
             impl toql::to_query::ToForeignQuery  for #struct_key_ident {
                fn to_foreign_query<M>(&self, toql_path :&str) ->toql::query::Query<M> {
                     let t = self;
                    toql::query::Query::<M>::new()
                    #(#toql_eq_foreign_predicates)*
                }
                fn slice_to_foreign_query<M>(entities: &[Self], toql_path :&str) ->toql::query::Query<M>
                where Self:Sized
                 {
                    let mut q = toql::query::Query::<M>::new();
                    for t in entities {
                        q = q.or_parentized(
                                toql::query::Query::<M>::new()
                                #(#toql_eq_foreign_predicates)*
                        );
                    };
                    q
                }
             }
             


            
            
           

            impl toql::key::Keyed for #rust_stuct_ident {
                type Key = #struct_key_ident;

                fn try_get_key(&self) -> toql::error::Result<Self::Key> {
                   Ok(  #struct_key_ident {
                       #( #key_getters),*
                       } )
                }
                fn try_set_key(&mut self, key: Self::Key) -> toql::error::Result<()> {
                  #( #key_setters;)*
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

            impl std::convert::TryFrom<Vec< toql::sql_arg::SqlArg>> for #struct_key_ident
            {
                type Error = toql::error::ToqlError;
                fn try_from(args: Vec< toql::sql_arg::SqlArg>) -> toql::error::Result<Self> {
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
                    <#rust_stuct_ident as toql::key::Keyed>::try_get_key(self).ok().hash(state);
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
