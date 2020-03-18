use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;
use heck::SnakeCase;

use crate::sane::{FieldKind, Struct};

pub(crate) struct GeneratedToqlQueryBuilder<'a> {
    rust_struct: &'a Struct,
    rust_struct_visibility: &'a syn::Visibility,
    builder_fields_struct: Ident,
    build_wildcard: bool,
    builder_fields: Vec<TokenStream>,
    key_composite_predicates: Vec<TokenStream>,
}

impl<'a> GeneratedToqlQueryBuilder<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedToqlQueryBuilder {

        let mut builder_fields : Vec<TokenStream> = Vec::new();

        for args in &toql.mapped_predicates {
            let rust_struct_visibility = &toql.rust_struct_visibility;
            
            let fnc_name= &args.name.to_snake_case();
            let fnc_ident = Ident::new(fnc_name, Span::call_site());
            let toql_field=  args.name.as_str().trim_start_matches("r#");
            builder_fields.push(quote!(
                        #rust_struct_visibility fn #fnc_ident (mut self) -> toql :: query :: Predicate {
                            self . 0 . push_str ( #toql_field ) ;
                            toql :: query :: Predicate :: from ( self . 0 )
                        }
                    ));

        }

        GeneratedToqlQueryBuilder {
            rust_struct: &toql,
            rust_struct_visibility: &toql.rust_struct_visibility,

            builder_fields_struct: syn::Ident::new(
                &format!("{}Fields", toql.rust_struct_name),
                Span::call_site(),
            ),
            build_wildcard: true,
            builder_fields,
            key_composite_predicates: Vec::new(),
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
                        .and(toql::query::Field::from(
                            format!("{}{}{}", path, if path.is_empty() || path.ends_with("_") {""}else {"_"}, #toql_field)
                        ).eq( &self . #key_index))
                    });
                }
            }
            x @ _ => {
                 let toql_field = &field.toql_field_name;
                if let FieldKind::Join(join_attrs) = x {
                   
                    if join_attrs.key {
                        self.key_composite_predicates.push(quote!(
                            .and( toql::query::QueryPredicate::predicate(&self. #key_index, #toql_field))
                            //.and(&self. #key_index)
                        ));
                    }
                }
                let toql_path = format!("{}_", toql_field);

                let path_fields_struct =
                    quote!( < #rust_type_ident as toql::query_builder::QueryFields>::FieldsType);

                self.builder_fields.push(quote!(
                                #rust_struct_visibility fn #rust_field_ident (mut self) -> #path_fields_struct {
                                    self.0.push_str(#toql_path);
                                    #path_fields_struct ::from_path(self.0)
                                }
                    ));
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

       /*  let key_predicate_code = quote!(
            let query = toql::query::Query::new() #(#key_composite_predicates)*;
            query
        ); */

        let builder = quote!(

                impl toql::query::QueryPredicate for &#struct_key_ident {
                    fn predicate(self, path :&str) -> toql::query::Query {
                        toql::query::Query::new()
                          #(#key_composite_predicates)*
                    } 
                }
          
                impl Into<toql::query::Query> for #struct_key_ident {
                    fn into(self) -> toql::query::Query {
                            toql::query::QueryPredicate::predicate(&self, "")
                    }
                }
                 impl Into<toql::query::Query> for &#struct_key_ident {
                    fn into(self) -> toql::query::Query {
                        toql::query::QueryPredicate::predicate(self, "")
                    }
                }
            impl toql::query_builder::QueryFields for #struct_ident {
                type FieldsType = #builder_fields_struct ;

                //fn predicates ( ) -> #builder_predicates_struct { #builder_predicates_struct :: new ( ) }
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
