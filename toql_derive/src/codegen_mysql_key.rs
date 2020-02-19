use crate::sane::FieldKind;
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct GeneratedMysqlKey<'a> {
    rust_struct: &'a crate::sane::Struct,

    forward_key_columns: usize,
    forward_key_joins: Vec<TokenStream>,
    mysql_deserialize_key: Vec<TokenStream>,
    mysql_forward_join_key: Vec<TokenStream>,
}

impl<'a> GeneratedMysqlKey<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlKey {
        GeneratedMysqlKey {
            rust_struct: &toql,
            forward_key_columns: 0,
            forward_key_joins: Vec::new(),
            mysql_deserialize_key: Vec::new(),
            mysql_forward_join_key: Vec::new(),
        }
    }

    pub fn add_key_deserialize(
        &mut self,
        field: &crate::sane::Field,
    ) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    return Ok(());
                }
                let error_field = format!(
                    "{}Key::{}",
                    &self.rust_struct.rust_struct_ident, rust_field_name
                );
               /*  let increment = if self.mysql_deserialize_key.is_empty() {
                    quote!(*i)
                } else {
                    quote!({
                        *i = *i + 1;
                        *i
                    })
                }; */
                self.mysql_deserialize_key.push(quote!(
                    (row.take_opt(*i).unwrap()
                                .map_err(|e| toql::error::ToqlError::DeserializeError(#error_field.to_string(), e.to_string())
                            )?, *i += 1).0
                ));
                self.forward_key_columns = self.forward_key_columns + 1;
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    return Ok(());
                }

                // Impl key from result row
                self.mysql_forward_join_key.push(quote!(
                   *i = < #rust_type_ident > ::forward_row(*i);
                ));
               /*  let increment = if self.mysql_deserialize_key.is_empty() {
                    quote!(i)
                } else {
                    quote!({
                        *i = *i + 1;
                        i
                    })
                }; */
                self.mysql_deserialize_key.push(quote!(
                     << #rust_type_ident as toql :: key :: Key > :: Key >:: from_row_with_index (row, i /*#increment*/)?
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

impl<'a> quote::ToTokens for GeneratedMysqlKey<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let _vis = &self.rust_struct.rust_struct_visibility;
        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;

        let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());

        let forward_key_columns = &self.forward_key_columns;
        let forward_key_joins = &self.forward_key_joins;
        let mysql_deserialize_key = &self.mysql_deserialize_key;

        let key = quote! {


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





        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_stuct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
