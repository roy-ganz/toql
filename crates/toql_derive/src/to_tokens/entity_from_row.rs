use crate::parsed::{
    field::{
        field_kind::FieldKind, join_field::JoinSelection, merge_field::MergeSelection,
        regular_field::RegularSelection,
    },
    parsed_struct::ParsedStruct,
};
use proc_macro2::TokenStream;
use std::collections::HashSet;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut deserialize_fields = Vec::new();
    let mut forwards = Vec::new();
    let mut impl_types = HashSet::new();
    let mut key_field_names = Vec::new();

    let struct_name_ident = &parsed_struct.struct_name;
    let struct_name = &parsed_struct.struct_name.to_string();

    for field in &parsed_struct.fields {
        let field_name = &field.field_name.to_string();
        let field_name_ident = &field.field_name;
        let field_base_type_path = &field.field_base_type;
        let error_field = format!("{}::{}", &struct_name_ident, field_name);

        match &field.kind {
            FieldKind::Skipped => {
                deserialize_fields.push(quote!( #field_name_ident : Default::default()));
            }
            FieldKind::Regular(regular_attrs) => {
                let field_name_ident = &field.field_name;

                impl_types.insert(field_base_type_path.to_owned());
                forwards.push(quote!(  <#field_base_type_path as toql::from_row::FromRow::<_,E>> :: forward (  &mut iter )?));

                // Check selection for optional Toql fields: Option<Option<..> or Option<..>
                match &regular_attrs.selection {
                    RegularSelection::SelectNullable => {
                        deserialize_fields.push(quote!(
                                #field_name_ident : {
                                    // TODO use peekable
                                    let mut it2 = iter.clone();
                                    if it2.next()
                                    .ok_or(toql::error::ToqlError::DeserializeError(
                                            toql::deserialize::error::DeserializeError::StreamEnd))?
                                    .is_selected() {
                                   Some(toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )?)
                                    } else {
                                        *iter = it2;
                                        None}
                                }
                        ));
                    }
                    RegularSelection::Select | RegularSelection::PreselectNullable => {
                        deserialize_fields.push(quote!(
                                #field_name_ident : {
                                    toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )?
                                }
                        ));
                    }
                    RegularSelection::Preselect => {
                        deserialize_fields.push(
                                 if regular_attrs.key {
                                    quote!(
                                        #field_name_ident : {
                                            match <#field_base_type_path as toql::from_row::FromRow::<_, E>>::from_row(row, i, iter)? {
                                                Some(s) => s,
                                                _ => return Ok(None),
                                            }
                                        }
                                    )

                                 } else {
                                 quote!(
                                        #field_name_ident : {
                                            toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )?
                                                    .ok_or(toql::deserialize::error::DeserializeError::SelectionExpected(#error_field.to_string()).into())?
                                        }
                                 )
                                 }
                                );
                    }
                };

                if regular_attrs.key {
                    key_field_names.push(field_name.to_string());
                }
            }
            FieldKind::Join(join_attrs) => {
                // Bound for joins
                impl_types.insert(field_base_type_path.to_owned());

                // For optional joined fields (left Joins) a discriminator field must be added to check
                // - for unselected entity (discriminator column is NULL Type)
                // - for null entity (discriminator column is false) - only left joins

                deserialize_fields.push(
                    match   join_attrs.selection {
                        JoinSelection::SelectLeft =>   //    Option<Option<T>>                 -> Selectable Nullable Join -> Left Join
                        quote!(
                                    #field_name_ident : {
                                          if iter.next().ok_or(toql::error::ToqlError::DeserializeError(
                                                toql::deserialize::error::DeserializeError::StreamEnd))?
                                                .is_selected()
                                        {
                                            //Some(< #field_type> :: from_row (  row , i, iter )?)
                                            let mut it2 = iter.clone();
                                            let n = i.clone();
                                            match toql::from_row::FromRow::<_,E> :: from_row(row, i, iter)? {
                                                Some(f) => Some(Some(f)),
                                                None => {
                                                    let s: usize =  <#field_base_type_path as toql::from_row::FromRow::<R,E>>::forward(&mut it2)?;
                                                    *iter = it2;
                                                    *i = n + s;
                                                    Some(None)
                                                }
                                            }
                                        } else {
                                            None
                                        }
                                    }
                            ),
                        JoinSelection::PreselectLeft =>   //    #[toql(preselect)] Option<T>  -> Nullable Join -> Left Join
                                quote!(
                                    #field_name_ident : {
                                        if !iter.next().ok_or(toql::error::ToqlError::DeserializeError(
                                            toql::deserialize::error::DeserializeError::StreamEnd))?
                                            .is_selected() {
                                            return Err(
                                                toql::error::ToqlError::DeserializeError(toql::deserialize::error::DeserializeError::SelectionExpected(#error_field.to_string())).into());
                                        }
                                        let mut it2 = iter . clone();
                                      let n = i . clone();
                                      // match <#field_type>::from_row(row, i, iter)? {
                                       match toql::from_row::FromRow::<_,E> ::from_row(row, i, iter)? {
                                                Some(f) => Some(f),
                                                None => {
                                                    let s: usize =  <#field_base_type_path as toql::from_row::FromRow::<R,E>>::forward(&mut it2)?;
                                                    *iter = it2;
                                                    *i = n + s;
                                                    None
                                                }
                                            }
                                    }
                                ),
                            JoinSelection::SelectInner =>  //    Option<T>                         -> Selectable Join -> Inner Join
                                        quote!(
                                        #field_name_ident : {
                                         if iter.next().ok_or(toql::error::ToqlError::DeserializeError(
                                            toql::deserialize::error::DeserializeError::StreamEnd))?
                                            .is_selected(){
                                            toql::from_row::FromRow::<_,E> :: from_row ( row , i, iter )?
                                         } else {
                                             None
                                         }
                                        }
                                    ),
                        JoinSelection::PreselectInner =>   //    T                                 -> Selected Join -> InnerJoin
                        if join_attrs.key {
                                    quote!(
                                        #field_name_ident : {
                                            if iter.next().ok_or(toql::error::ToqlError::DeserializeError(
                                                toql::deserialize::error::DeserializeError::StreamEnd))?
                                                .is_selected()
                                            {
                                                match toql::from_row::FromRow::<_,E> :: from_row (  row , i, iter )? {
                                                    Some(s) => s,
                                                    None => return Err(toql::error::ToqlError::ValueMissing(#field_name.to_string()).into()),
                                                }
                                            } else {
                                                return Err(toql::error::ToqlError::DeserializeError(
                                                    toql::deserialize::error::DeserializeError::SelectionExpected(#field_name.to_string())).into());
                                            }
                                        }
                                    )

                                 } else {
                                    quote!(
                                        #field_name_ident : {
                                            let err = toql::error::ToqlError::DeserializeError(toql::deserialize::error::DeserializeError::SelectionExpected(#error_field.to_string()));
                                            if iter.next().ok_or(
                                                toql::error::ToqlError::DeserializeError(
                                                toql::deserialize::error::DeserializeError::StreamEnd))?.is_selected(){
                                                    toql::from_row::FromRow::<_,E> :: from_row ( row , i, iter )?.ok_or(err)?
                                            } else {
                                                return Err(err.into());
                                            }
                                        }
                                    )
                                 }
                    }
                );
                forwards.push(
                        quote!{
                            if iter.next().ok_or(
                                toql::error::ToqlError::DeserializeError(toql::deserialize::error::DeserializeError::StreamEnd))?
                                .is_selected()
                            {
                                < #field_base_type_path as toql::from_row::FromRow::<R,E>> :: forward (  &mut iter )?
                            } else {
                                0
                            }
                        });
                if join_attrs.key {
                    key_field_names.push(field_name.to_string());
                }
            }
            FieldKind::Merge(merge_attrs) => {
                // Merge doesn't need bounds, as they are not deserialized within toql::from_row::FromRow
                deserialize_fields.push(if merge_attrs.selection == MergeSelection::Select {
                    quote!( #field_name_ident : None)
                } else {
                    quote!( #field_name_ident : Vec::new())
                });
            }
        }
    }

    // Generate stream
    let struct_ident = struct_name_ident;

    let impl_types = &impl_types
        .iter()
        .map(|k| quote!( #k :toql::from_row::FromRow<R,E>, ))
        .collect::<Vec<_>>();

    let code = quote!(
        impl<R,E> toql::from_row::FromRow<R, E> for #struct_ident
        where  E: std::convert::From<toql::error::ToqlError>,
          #(#impl_types)*
        {
            fn forward<'a, I>( mut iter: &mut I) -> std::result::Result<usize, E>
            where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> {
                Ok(0 #(+ #forwards)*)
            }

            #[allow(unused_variables, unused_mut, unused_imports)]
            fn from_row<'a, I> ( mut row : &R , i : &mut usize, mut iter: &mut I)
                ->std::result:: Result < Option<#struct_ident>, E>
                where I:   Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone {
                use toql::sql_builder::select_stream::Select;

                Ok ( Some(#struct_ident {
                    #(#deserialize_fields),*

                }))
            }
        }
    );

    log::debug!("Source code for `{}`:\n{}", &struct_name, code.to_string());

    tokens.extend(code);
}
