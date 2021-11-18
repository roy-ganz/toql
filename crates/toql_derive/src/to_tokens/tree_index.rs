use crate::parsed::{field::field_kind::FieldKind, parsed_struct::ParsedStruct};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut index_type_bounds = Vec::new();
    let mut dispatch_index_code = Vec::new();
    let mut dispatch_types = HashSet::new();

    let struct_name = &parsed_struct.struct_name.to_string();
    let struct_name_ident = &parsed_struct.struct_name;
    let struct_key_ident = Ident::new(&format!("{}Key", &struct_name), Span::call_site());

    for field in &parsed_struct.fields {
        let field_type = &field.field_type;
        let toql_field_name = &field.toql_query_name;
        let field_base_type = &field.field_base_type;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(_) => {}
            FieldKind::Join(_) => {
                dispatch_types.insert(field_base_type.to_owned());
                dispatch_index_code.push(quote!(
                        #toql_field_name => {
                            <#field_base_type as toql::tree::tree_index::TreeIndex<R,E>>::
                            index(descendents, rows, row_offset, index)?
                        }
                ));
                index_type_bounds.push(quote!(
                    #field_type : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #field_type as toql :: from_row :: FromRow < R >> :: Error>
                    ));
            }
            FieldKind::Merge(_) => {
                dispatch_types.insert(field_base_type.to_owned());
                dispatch_index_code.push(quote!(
                       #toql_field_name => {
                             <#field_base_type as toql::tree::tree_index::TreeIndex<R,E>>::
                            index(descendents, rows, row_offset, index)?
                       }
                ));
                index_type_bounds.push(quote!(
                    <#field_type as toql::keyed::Keyed>::Key : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < <#field_base_type as toql::keyed::Keyed>::Key as toql :: from_row :: FromRow < R >> :: Error>
                    ));
            }
        };
    }

    let tree_index_dispatch_bounds = dispatch_types
        .iter()
        .map(|t| quote!( #t :  toql::tree::tree_index::TreeIndex<R, E>,))
        .collect::<Vec<_>>();
    let tree_index_dispatch_bounds_ref = tree_index_dispatch_bounds.clone();

    // Generate token stream
    let mods = quote! {

        impl<R,E> toql::tree::tree_index::TreeIndex<R, E> for #struct_name_ident
        where  E: std::convert::From<toql::error::ToqlError>,
            #struct_key_ident: toql::from_row::FromRow<R, E>,
            #(#tree_index_dispatch_bounds)*
        {
            #[allow(unused_variables, unused_mut)]
            fn index<'a, I>( mut descendents:  I,
                rows: &[R], row_offset: usize, index: &mut std::collections::HashMap<u64,Vec<usize>>)
                -> std::result::Result<(), E>
                where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {

                use toql::from_row::FromRow;
                use std::hash::Hash;
                use std::hash::Hasher;
                use std::collections::hash_map::DefaultHasher;
                use toql::sql_builder::select_stream::Select;

                match descendents.next() {

                    Some(d) => {
                        match d.as_str() {
                            #(#dispatch_index_code),*
                            f @ _ => {
                                return Err(
                                    toql::error::ToqlError::SqlBuilderError (
                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                        .into());
                            }
                        }
                    },
                    None => {
                            for (n, row) in rows.into_iter().enumerate() {
                                let mut iter = std::iter::repeat(&Select::Query);
                                let mut  i= row_offset;
                                let fk = #struct_key_ident ::from_row(&row, &mut i, &mut iter)?
                                    .ok_or(toql::error::ToqlError::ValueMissing(
                                                    <#struct_key_ident as toql::key::Key>::columns().join(", ")
                                                ))?; // Skip Primary key

                                let mut s = DefaultHasher::new();
                                fk.hash(&mut s);
                                let fk_hash =  s.finish();

                                index.entry(fk_hash)
                                .and_modify(|h| h.push(n))
                                .or_insert(vec![n]);
                            }
                        }
                }
                Ok(())
            }
        }

        impl<R,E> toql::tree::tree_index::TreeIndex<R, E> for &#struct_name_ident
        where  E: std::convert::From<toql::error::ToqlError>,
            #struct_key_ident: toql::from_row::FromRow<R, E>,
            #(#tree_index_dispatch_bounds_ref)*
        {
            #[allow(unused_mut)]
            fn index<'a, I>( mut descendents: I,
                rows: &[R], row_offset: usize, index: &mut std::collections::HashMap<u64,Vec<usize>>)
                -> std::result::Result<(), E>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                    <#struct_name_ident as  toql::tree::tree_index::TreeIndex<R,E>>::index(descendents,  rows, row_offset, index)
                }
        }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
