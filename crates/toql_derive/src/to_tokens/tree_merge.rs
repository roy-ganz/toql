use crate::parsed::{
    field::{field_kind::FieldKind, join_field::JoinSelection, merge_field::MergeSelection},
    parsed_struct::ParsedStruct,
};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut merge_type_bounds = Vec::new();
    let mut dispatch_merge_code = Vec::new();
    let mut merge_code = Vec::new();
    let mut dispatch_types = HashSet::new();

    let struct_name = &parsed_struct.struct_name.to_string();
    let struct_name_ident = &parsed_struct.struct_name;

    let struct_key_ident = Ident::new(&format!("{}Key", &struct_name), Span::call_site());

    for field in &parsed_struct.fields {
        let field_name_ident = &field.field_name;
        let field_name = &field.field_name.to_string();
        let field_type = &field.field_type;
        let field_type_string = format!("{:?}", &field.field_type);
        let toql_query_name = &field.toql_query_name;
        let field_base_type = &field.field_base_type;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(_) => {}
            FieldKind::Join(join_attrs) => {
                dispatch_types.insert(field_base_type.to_owned());

                dispatch_merge_code.push(
                    match &join_attrs.selection {
                        JoinSelection::SelectLeft =>  {
                            quote!(
                                #toql_query_name => {
                                  let value =  self. #field_name_ident .as_mut().ok_or(
                                    toql::error::ToqlError::ValueMissing( #field_type_string.to_string()),
                                    )?;
                                    if let Some(val) =  value.as_mut() {
                                        toql::tree::tree_merge::TreeMerge::merge( val,
                                            descendents, &field, rows, row_offset, index, selection_stream)?
                                    }
                                }
                            )
                        },
                        JoinSelection::PreselectLeft  =>  {
                            // #[toql(preselect)] Option<T>
                            quote!(
                                #toql_query_name => {
                                    if let Some(val) =  self. # field_name_ident.as_mut() {
                                        toql::tree::tree_merge::TreeMerge::merge( val,
                                            descendents, &field, rows, row_offset, index, selection_stream)?
                                    }
                                }
                            )
                        },
                      JoinSelection::PreselectInner => {
                            quote!(
                                #toql_query_name => {
                                    toql::tree::tree_merge::TreeMerge::merge(&mut self. #field_name_ident,
                                    descendents, &field, rows, row_offset, index, selection_stream)?
                                }
                            )
                        }
                       JoinSelection::SelectInner=> {
                            let unwrap_mut =
                                    quote!(.as_mut().ok_or(toql::error::ToqlError::ValueMissing(#field_name.to_string()))?);
                           quote!(
                                #toql_query_name => {
                                    toql::tree::tree_merge::TreeMerge::merge( self. #field_name_ident #unwrap_mut,
                                        descendents, &field, rows, row_offset, index, selection_stream)?
                                }
                            )
                        }
                    }
               );
                merge_type_bounds.push(quote!(
                    #field_type : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #field_type as toql :: from_row :: FromRow < R >> :: Error>
                    ));
            }
            FieldKind::Merge(merge) => {
                dispatch_types.insert(field_base_type.to_owned());
                let refer_mut = if merge.selection == MergeSelection::Select {
                    quote!()
                } else {
                    quote!(&mut)
                };
                let unwrap_mut = if merge.selection == MergeSelection::Select {
                    quote!(.as_mut().ok_or(toql::error::ToqlError::ValueMissing(#field_name.to_string()))?)
                } else {
                    quote!()
                };

                dispatch_merge_code.push(
                   quote!(
                        #toql_query_name => {
                            for f in #refer_mut self. #field_name_ident #unwrap_mut {
                                toql::tree::tree_merge::TreeMerge::merge(f,
                                    descendents.clone(), &field, rows, row_offset, index, selection_stream)?
                            }
                        }
                    )
               );

                merge_type_bounds.push(quote!(
                    <#field_type as toql::keyed::Keyed>::Key : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < <#field_type as toql::keyed::Keyed>::Key as toql :: from_row :: FromRow < R >> :: Error>,
                    #field_type : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #field_type as toql :: from_row :: FromRow < R >> :: Error>
                    ));

                let merge_push = if merge.selection == MergeSelection::Select {
                    quote!(
                        self. #field_name_ident .as_mut().unwrap() .push(e);
                    )
                } else {
                    quote!(self. #field_name_ident .push(e);)
                };
                let empty_vector_init = if merge.selection == MergeSelection::Select {
                    quote!(
                    if self. #field_name_ident .is_none() {
                            self. #field_name_ident = Some(Vec::new())
                        })
                } else {
                    quote!()
                };

                merge_code.push(
                    quote!(

                        #toql_query_name  => {
                            #empty_vector_init
                            for row_number in row_numbers {
                                let mut i = n;
                                let mut iter = std::iter::repeat(&Select::Query);
                                let row: &R = &rows[*row_number];
                                let fk = #struct_key_ident::from_row(&row, &mut i, &mut iter)?
                                    .ok_or(toql::error::ToqlError::ValueMissing( #toql_query_name .to_string()))?;
                                if fk ==  pk {
                                    let mut iter = selection_stream.iter();
                                    let e = #field_base_type::from_row(&row, &mut i, &mut iter)?
                                        .ok_or(toql::error::ToqlError::ValueMissing( #toql_query_name .to_string()))?;
                                    #merge_push
                                }
                            }
                        },
                    )
                );
            }
        };
    }

    // Build token stream
    let tree_merge_dispatch_bounds = dispatch_types
        .iter()
        .map(|t| {
            quote!(
            #t : toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
            )
        })
        .collect::<Vec<_>>();
    let tree_merge_dispatch_bounds_ref = tree_merge_dispatch_bounds.clone();

    let mods = quote! {

        impl<R,E> toql::tree::tree_merge::TreeMerge<R,E> for #struct_name_ident
        where  E: std::convert::From<toql::error::ToqlError>,
        #struct_key_ident: toql::from_row::FromRow<R, E>,
        #(#tree_merge_dispatch_bounds)*
        {
            #[allow(unreachable_code, unused_variables, unused_mut,unused_imports)]
            fn merge<'a, I>(  &mut self, mut descendents: I, field: &str,
                rows: &[R],row_offset: usize, index: &std::collections::HashMap<u64,Vec<usize>>,
                selection_stream: &toql::sql_builder::select_stream::SelectStream)
            -> std::result::Result<(), E>
            where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
            {
                use toql::keyed::Keyed;
                use toql::from_row::FromRow;
                use std::hash::Hash;
                use std::hash::Hasher;
                use std::collections::hash_map::DefaultHasher;
                use toql::sql_builder::select_stream::Select;

                match descendents.next() {

                    Some(d) => {
                        match d.as_str() {
                            #(#dispatch_merge_code),*
                            f @ _ => {
                                return Err(
                                    toql::error::ToqlError::SqlBuilderError(
                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                        .into());
                            }
                        }
                    },
                    None => {
                            let pk : #struct_key_ident = <Self as toql::keyed::Keyed>::key(&self); // removed .into()
                            let mut s = DefaultHasher::new();
                            pk.hash(&mut s);
                            let h = s.finish();
                            let default_vec: Vec<usize>= Vec::new();
                            let row_numbers : &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                            let  n = row_offset;

                            match field {
                                #(#merge_code)*
                                f @ _ => {
                                    return Err(
                                            toql::error::ToqlError::SqlBuilderError(
                                                toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                            .into());
                                }
                            };
                    }
                }
                Ok(())
            }
        }

        impl<R,E> toql::tree::tree_merge::TreeMerge<R,E> for &mut #struct_name_ident
        where  E: std::convert::From<toql::error::ToqlError>,
        #struct_key_ident: toql::from_row::FromRow<R, E>,
        #(#tree_merge_dispatch_bounds_ref)*
        {
            #[allow(unused_mut)]
            fn merge<'a, I>(  &mut self, mut descendents: I, field: &str,
                    rows: &[R],row_offset: usize, index: &std::collections::HashMap<u64,Vec<usize>>,
                    selection_stream: &toql::sql_builder::select_stream::SelectStream)
            -> std::result::Result<(), E>
            where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
            {
                <#struct_name_ident as toql::tree::tree_merge::TreeMerge<R,E>>::merge(self, descendents, field, rows, row_offset, index, selection_stream)
            }
        }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
