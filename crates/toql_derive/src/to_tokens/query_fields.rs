use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::parsed::{field::field_kind::FieldKind, parsed_struct::ParsedStruct};

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut builder_fields = Vec::new();
    let mut key_composite_predicates = Vec::new();

    let struct_vis = &parsed_struct.vis;
    for (name, _arg) in &parsed_struct.predicates {
        let fnc_name = &name.to_snake_case();
        let fnc_ident = Ident::new(fnc_name, Span::call_site());
        let toql_field = name.as_str().trim_start_matches("r#");
        builder_fields.push(quote!(
            #struct_vis fn #fnc_ident (mut self) -> toql :: query :: predicate :: Predicate {
                self . 0 . push_str ( #toql_field ) ;
                toql :: query :: predicate :: Predicate :: from ( self . 0 )
            }
        ));
    }

    for field in &parsed_struct.fields {
        if field.skip {
            continue;
        }

        let field_name_ident = &field.field_name;

        let key_index = syn::Index::from(key_composite_predicates.len());
        match &field.kind {
            FieldKind::Regular(ref regular_kind) => {
                let toql_field = &field.toql_query_name;

                builder_fields.push(quote!(
                    #struct_vis fn #field_name_ident (mut self) -> toql :: query :: field::  Field {
                        self . 0 . push_str ( #toql_field ) ;
                        toql :: query :: field:: Field :: from ( self . 0 )
                    }
                ));
                if regular_kind.key {
                    key_composite_predicates.push(quote! {
                        .and(toql::query::field::Field::from(
                            format!("{}{}{}", path, if path.is_empty() || path.ends_with("_") {""}else {"_"}, #toql_field)
                        ).eq( &self . #key_index))
                    });
                }
            }
            x => {
                let toql_field = &field.toql_query_name;
                if let FieldKind::Join(join_attrs) = x {
                    if join_attrs.key {
                        key_composite_predicates.push(quote!(
                            .and( toql::query::QueryPredicate::predicate(&self. #key_index, #toql_field))
                            //.and(&self. #key_index)
                        ));
                    }
                }
                let toql_path = format!("{}_", toql_field);
                let field_base_type = &field.field_base_type;
                let path_fields_struct =
                    quote!( < #field_base_type as toql::query_fields::QueryFields>::FieldsType);

                builder_fields.push(quote!(
                            #struct_vis fn #field_name_ident (mut self) -> #path_fields_struct {
                                self.0.push_str(#toql_path);
                                #path_fields_struct ::from_path(self.0)
                            }
                ));
            }
        };
    }
    // Generate token stream
    let builder_fields = &builder_fields;
    let struct_name_ident = &parsed_struct.struct_name;
    let builder_fields_struct =
        syn::Ident::new(&format!("{}Fields", struct_name_ident), Span::call_site());
    let builder = quote!(
        impl toql::query_fields::QueryFields for #struct_name_ident {
            type FieldsType = #builder_fields_struct ;
            fn fields ( ) -> #builder_fields_struct { #builder_fields_struct :: new ( ) }
            fn fields_from_path ( path : String ) -> #builder_fields_struct { #builder_fields_struct :: from_path ( path ) }
        }

        #struct_vis struct #builder_fields_struct ( String ) ;

        impl toql::query_path::QueryPath for #builder_fields_struct {
            fn into_path(self) -> String {
            self.0
            }
        }

        impl #builder_fields_struct {
            #struct_vis fn new ( ) -> Self { Self :: from_path ( String :: from ( "" ) ) }
            #struct_vis fn from_path ( path : String ) -> Self { Self ( path ) }
            #struct_vis fn into_name ( self) -> String { self.0}
            #(#builder_fields)*

        }
    );

    log::debug!(
        "Source code for `{}`:\n{}",
        &struct_name_ident,
        builder.to_string()
    );

    tokens.extend(builder);
}
