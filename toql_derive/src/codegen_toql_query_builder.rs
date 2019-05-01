

use crate::annot::Toql;
use crate::annot::ToqlField;
use quote::quote;

use proc_macro2::Span;

use heck::MixedCase;
use heck::SnakeCase;
use syn::Ident;

pub(crate) struct GeneratedToqlQueryBuilder<'a> {
    struct_ident: &'a Ident,
    vis: &'a syn::Visibility,
    sql_table_name: String,
    sql_table_alias: String,
    builder_fields_struct: Ident,
    builder_fields: Vec<proc_macro2::TokenStream>,
  
}

impl<'a> GeneratedToqlQueryBuilder<'a> {
    pub(crate) fn from_toql(toql: &Toql) -> GeneratedToqlQueryBuilder {
       
       let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        GeneratedToqlQueryBuilder {
            struct_ident: &toql.ident,
            vis: &toql.vis,
            sql_table_name: toql.table.clone().unwrap_or(renamed_table), //toql.ident.to_string(),
            sql_table_alias: toql
                .alias
                .clone()
                .unwrap_or(toql.ident.to_string().to_snake_case()), //  toql.ident.to_string().to_snake_case(),
            builder_fields_struct: syn::Ident::new(
                &format!("{}Fields", toql.ident.to_string()),
                Span::call_site(),
            ),
            builder_fields: Vec::new(),
           
        }
    }

    
    pub(crate) fn add_field_for_builder(&mut self, _toql: &Toql, field: &'a ToqlField) {
        let field_ident = &field.ident;
        let vis = &_toql.vis;
        if field.join.is_empty() && field.merge.is_empty() {
            let toql_field = format!("{}", field_ident.as_ref().unwrap()).to_mixed_case();
            self.builder_fields.push(quote!(
                #vis fn #field_ident (mut self) -> toql :: query :: Field {
                    self . 0 . push_str ( #toql_field ) ;
                    toql :: query :: Field :: from ( self . 0 )
                }
            ));
        } else {
            let toql_field = format!(
                "{}_",
                format!("{}", field_ident.as_ref().unwrap()).to_mixed_case()
            );

            let field_type = field.first_non_generic_type().unwrap();

           /*  let path_ident =
                syn::Ident::new(&field_type.to_string().to_snake_case(), Span::call_site()); */
            let type_ident: &Ident = field_type;
           /*  let field_type_ident =
                syn::Ident::new(&format!("{}Fields", type_ident), Span::call_site()); */
            let path_fields_struct =  quote!( < #type_ident as toql::query_builder::FieldsType>::FieldsType); //quote!( super :: #path_ident :: #field_type_ident );

            self.builder_fields.push(quote!(
                        #vis fn #field_ident (mut self) -> #path_fields_struct {
                            self.0.push_str(#toql_field);
                            #path_fields_struct ::from_path(self.0)
                        }
            ));
        }
    }
}

impl<'a> quote::ToTokens for GeneratedToqlQueryBuilder<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
      
        let vis = self.vis;        
        let builder_fields_struct = &self.builder_fields_struct;
        let builder_fields = &self.builder_fields;
        let struct_ident = &self.struct_ident;

        let builder = quote!(

            impl toql::query_builder::FieldsType for #struct_ident {
                type FieldsType = #builder_fields_struct ;
            }

            impl #struct_ident {
                #vis fn fields ( ) -> #builder_fields_struct { #builder_fields_struct :: new ( ) }
                #vis fn fields_from_path ( path : String ) -> #builder_fields_struct { #builder_fields_struct :: from_path ( path ) }
            }


            #vis struct #builder_fields_struct ( String ) ;
            impl #builder_fields_struct {
                #vis fn new ( ) -> Self { Self :: from_path ( String :: from ( "" ) ) }
                #vis fn from_path ( path : String ) -> Self { Self ( path ) }
                #(#builder_fields)*
            }
        );
        
        log::debug!("Source code for `{}`:\n{}", &self.struct_ident, builder.to_string());
        
        tokens.extend(builder);
    }
}
