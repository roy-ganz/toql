use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::sane::{FieldKind, Struct};

pub(crate) struct GeneratedToqlQueryBuilder<'a> {
    //struct_ident: &'a Ident,
    rust_struct: &'a Struct,
    rust_struct_visibility: &'a syn::Visibility,
    builder_fields_struct: Ident,
    build_wildcard: bool,
    builder_fields: Vec<TokenStream>,
    //key_predicates: Vec<TokenStream>,
    key_simple_predicate: Option<TokenStream>,
    key_composite_predicates: Vec<TokenStream>,
     serde_key: bool
}

impl<'a> GeneratedToqlQueryBuilder<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlQueryBuilder {
        GeneratedToqlQueryBuilder {
            rust_struct: &toql,
            rust_struct_visibility: &toql.rust_struct_visibility,

            builder_fields_struct: syn::Ident::new(
                &format!("{}Fields", toql.rust_struct_name),
                Span::call_site(),
            ),
            build_wildcard: true,
            builder_fields: Vec::new(),
            key_simple_predicate: None,
            key_composite_predicates: Vec::new(),
            serde_key: toql.serde_key
        }
    }

    pub(crate) fn add_field_for_builder(&mut self, field: &crate::sane::Field) {
        let rust_field_ident = &field.rust_field_ident;
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_struct_visibility = &self.rust_struct_visibility;

        // Omit wildcard function, if there is already a field called `wildcard`
        if rust_field_name == "wildcard" {
            self.build_wildcard = false;
        }
        let key_index = syn::Index::from(self.key_composite_predicates.len());
        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                let toql_field = &field.toql_field_name;

                self.builder_fields.push(quote!(
                        #rust_struct_visibility fn #rust_field_ident (mut self) -> toql :: query :: Field {
                            self . 0 . push_str ( #toql_field ) ;
                            toql :: query :: Field :: from ( self . 0 )
                        }
                    ));
                if regular_attrs.key {
                    self.key_composite_predicates.push(quote! {
                        .and(toql::query::Field::from(#toql_field).eq( &key . #key_index))
                    });
                    
                    self.key_simple_predicate = Some( quote!(
                            .and(toql::query::Field::from(#toql_field).ins( it.map(|k| k. #key_index).collect()))
                        ));
                    } 
                },
            
            x @ _ => {
                if let FieldKind::Join(join_attrs) = x {
                    if join_attrs.key {
                        self.key_composite_predicates.push(
                            quote!(
                                    .and(key. #key_index)
                                    //.and( <#rust_type_ident as toql::key::Key>::Key::key_predicate(key. #key_index))
                                )
                        );
                        self.key_simple_predicate = Some( quote!(
                             .and( {
                                let q = toql::query::Query::new();
                                for key in it {
                                    q = q.and( key. #key_index);
                                    //q = q.and( <#rust_type_ident as toql::key::Key>::Key::key_predicate(key. #key_index));
                                }
                                q
                            })
                        ));
                     
                    
                    }

                    let toql_path = format!("{}_", field.toql_field_name);

                    let path_fields_struct =
                        quote!( < #rust_type_ident as toql::query_builder::QueryFields>::FieldsType);

                    self.builder_fields.push(quote!(
                                #rust_struct_visibility fn #rust_field_ident (mut self) -> #path_fields_struct {
                                    self.0.push_str(#toql_path);
                                    #path_fields_struct ::from_path(self.0)
                                }
                    ));
            }
            }
        };
    }
}

impl<'a> quote::ToTokens for GeneratedToqlQueryBuilder<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let rust_struct_visibility = self.rust_struct_visibility;
        let builder_fields_struct = &self.builder_fields_struct;
        let builder_fields = &self.builder_fields;
        let struct_ident = &self.rust_struct.rust_struct_ident;

       // let key_predicates = &self.key_predicates;
        let struct_key_ident = Ident::new(&format!("{}Key", &struct_ident), Span::call_site());

        let wildcard = if self.build_wildcard {
            quote!(
                pub fn wildcard( self) -> toql::query::Wildcard {
                    toql::query::Wildcard::from(self.0)
                }
            )
        } else {
            quote!()
        };

        let key_composite_predicates = &self.key_composite_predicates;
       

        let key_predicate_code = if key_composite_predicates.len() == 1 {
             let key_simple_predicate = self.key_simple_predicate.as_ref().unwrap();
               quote!(
                  toql::query::Query::new() #key_simple_predicate
               ) 
        } else {
            quote!(
                let mut query = toql::query::Query::new();
                for key in it{
                    let q = toql::query::Query::new() #(#key_composite_predicates)*;
                    query = query.or(q);
                }
                query
            )
        };
        let struct_key_wrapper_ident = Ident::new(&format!("{}Keys", &struct_ident), Span::call_site());

    let serde = if self.serde_key {
            quote!( ,Deserialize, Serialize)
        } else { 
            quote!()
        };

        let builder = quote!(



                impl Into<toql::query::Query> for #struct_key_ident {
                    fn into(self) -> toql::query::Query {
                            #struct_key_wrapper_ident( vec![self]).into()
                    }
                }
                impl Into<toql::query::Query> for &#struct_key_ident {
                    fn into(self) -> toql::query::Query {
                             self.to_owned().into()
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
                impl IntoIterator for #struct_key_wrapper_ident {
                    type Item = #struct_key_ident;
                    type IntoIter = ::std::vec::IntoIter<Self::Item>;

                    fn into_iter(self) -> Self::IntoIter {
                        self.0.into_iter()
                    }
                }

                impl Into<toql::query::Query> for #struct_key_wrapper_ident {
                    fn into(self) -> toql::query::Query {
                         let it = self.0.into_iter();
                        #key_predicate_code
                    }
                } 
                 impl Into<toql::query::Query> for &#struct_key_wrapper_ident {
                    fn into(self) -> toql::query::Query {
                       
                       let it  =  self.0.clone().into_iter(); // Sorry, RUST just burned too much time on this.
                        #key_predicate_code
                    }
                } 
                
         

            impl toql::query_builder::QueryFields for #struct_ident {
                type FieldsType = #builder_fields_struct ;

                fn fields ( ) -> #builder_fields_struct { #builder_fields_struct :: new ( ) }
                fn fields_from_path ( path : String ) -> #builder_fields_struct { #builder_fields_struct :: from_path ( path ) }
            }


            #rust_struct_visibility struct #builder_fields_struct ( String ) ;
            impl #builder_fields_struct {
                #rust_struct_visibility fn new ( ) -> Self { Self :: from_path ( String :: from ( "" ) ) }
                #rust_struct_visibility fn from_path ( path : String ) -> Self { Self ( path ) }
                #(#builder_fields)*

                #wildcard
            }
        );

        log::debug!(
            "Source code for `{}`:\n{}",
            &self.rust_struct.rust_struct_name,
            builder.to_string()
        );

        tokens.extend(builder);
    }
}
