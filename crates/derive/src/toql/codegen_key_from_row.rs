use crate::sane::FieldKind;
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct CodegenKeyFromRow<'a> {
    rust_struct: &'a crate::sane::Struct,

    forward_key_columns: usize,
    deserialize_key: Vec<TokenStream>,
    forward_join_key: Vec<TokenStream>,
}

impl<'a> CodegenKeyFromRow<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenKeyFromRow {
        CodegenKeyFromRow {
            rust_struct: &toql,
            forward_key_columns: 0,
            deserialize_key: Vec::new(),
            forward_join_key: Vec::new(),
        }
    }

    pub fn add_key_deserialize(
        &mut self,
        field: &crate::sane::Field,
    ) -> darling::error::Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_field_ident = &field.rust_field_ident;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    return Ok(());
                }
                let error_field = format!(
                    "{}Key::{}",
                    &self.rust_struct.rust_struct_ident, rust_field_name
                );
               /*  let increment = if self.deserialize_key.is_empty() {
                    quote!(*i)
                } else {
                    quote!({
                        *i = *i + 1;
                        *i
                    })
                }; */
                self.deserialize_key.push(quote!(
                    #rust_field_ident: ($col_get!(row, *i)
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
                self.forward_join_key.push(quote!(
                   *i = < #rust_type_ident > ::skip(*i);
                ));

              
                self.deserialize_key.push(quote!(
                    
                    #rust_field_ident: << #rust_type_ident as toql :: key :: Keyed > :: Key >:: from_row_with_index (row, i, iter /*#increment*/)?
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

impl<'a> quote::ToTokens for CodegenKeyFromRow<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let _vis = &self.rust_struct.rust_struct_visibility;
        let rust_stuct_ident = &self.rust_struct.rust_struct_ident;

        let struct_key_ident = Ident::new(&format!("{}Key", &rust_stuct_ident), Span::call_site());
      
        let deserialize_key = &self.deserialize_key;
       
        let key = quote! {
                
                    impl<R,E> toql::from_row::FromRow<R, E> for #struct_key_ident 
                    // TODO BOUNDS
                    {
                                    
                            #[allow(unused_variables, unused_mut)]
                            fn from_row_with_index<'a, I> ( mut row : &E , i : &mut usize, mut iter: &mut I)
                                -> std::result:: Result < #struct_key_ident, E> 
                                where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> {

                                Ok ( #struct_key_ident{
                                    #(#deserialize_key),*
                                })
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
