use crate::parsed::{field::field_kind::FieldKind, parsed_struct::ParsedStruct};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut deserialize_key = Vec::new();
    let mut forwards = Vec::new();
    let mut regular_types = HashSet::new();
    let mut join_types = HashSet::new();

    for field in &parsed_struct.fields {
        let field_type = &field.field_type;
        let field_name = &field.field_name.to_string();
        let field_name_ident = &field.field_name;
        let field_base_type = &field.field_base_type;

        let error_field = format!("{}Key::{}", &parsed_struct.struct_name, field_name);

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if !regular_attrs.key {
                    continue;
                }
                regular_types.insert(field.field_base_type.to_owned());
                deserialize_key.push(quote!(
                        #field_name_ident: {
                                    toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )?
                                        .ok_or(toql::deserialize::error::DeserializeError::SelectionExpected(
                                        #error_field.to_string()).into())?
                        }
                    ));
                forwards.push(quote!( <#field_base_type as toql::from_row::FromRow::<R,E>> :: forward ( &mut iter )?) )
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    continue;
                }

                join_types.insert(field.field_base_type.to_owned());
                // Impl key from result row
                forwards.push(quote!(
                    <#field_base_type as toql::from_row::FromRow::<R,E>> :: forward ( &mut iter )?
                ));

                deserialize_key.push(quote!(
                        #field_name_ident: {
                            << #field_type as toql :: keyed :: Keyed > :: Key >:: from_row(row, i, iter)?
                                    .ok_or(toql::error::ToqlError::ValueMissing(#error_field.to_string()))?
                        }
                    ));
            }
            _ => {}
        }
    }

    let _vis = &parsed_struct.vis;
    let struct_name_ident = &parsed_struct.struct_name;

    let struct_key_ident = Ident::new(&format!("{}Key", &struct_name_ident), Span::call_site());

    let regular_types = regular_types
        .iter()
        .map(|k| quote!( #k :toql::from_row::FromRow<R,E>, ))
        .collect::<Vec<_>>();
    let join_types = join_types
        .iter()
        .map(|k| {
            quote!(
            #k :  toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
            <#k as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
            )
        })
        .collect::<Vec<_>>();

    let key = quote! {

                impl<R,E> toql::from_row::FromRow<R, E> for #struct_key_ident
                where  E: std::convert::From<toql::error::ToqlError>,
                 #(#regular_types)*
                 #(#join_types)*

                {
                        fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
                        where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select>{

                           Ok(0 #(+ #forwards)* )
                        }

                        #[allow(unused_variables, unused_mut)]
                        fn from_row<'a, I> ( mut row : &R , i : &mut usize, mut iter: &mut I)
                            -> std::result:: Result < Option<#struct_key_ident>, E>
                            where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone {

                            Ok ( Some(#struct_key_ident{
                                #(#deserialize_key),*
                            }))
                        }

            }

    };

    log::debug!(
        "Source code for `{}`:\n{}",
        struct_name_ident,
        key.to_string()
    );
    tokens.extend(key);
}
