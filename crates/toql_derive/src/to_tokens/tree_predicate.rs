use crate::parsed::{
    field::{field_kind::FieldKind, join_field::JoinSelection, merge_field::MergeSelection},
    parsed_struct::ParsedStruct,
};
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut dispatch_predicate_args_code = Vec::new();
    let mut dispatch_predicate_columns_code = Vec::new();

    let struct_name = &parsed_struct.struct_name.to_string();
    let struct_name_ident = &parsed_struct.struct_name;

    for field in &parsed_struct.fields {
        let field_name_ident = &field.field_name;
        let field_name = &field.field_name.to_string();
        let toql_query_name = &field.toql_query_name;
        let field_base_type = &field.field_base_type;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(_) => {}
            FieldKind::Join(join_attrs) => {
                dispatch_predicate_args_code.push(
                    match &join_attrs.selection {
                        JoinSelection::SelectLeft => {
                            // Option<Option<T>>
                            quote!(
                                #toql_query_name => {
                                    let value= self. #field_name_ident .as_ref().ok_or(
                                toql::error::ToqlError::ValueMissing( #toql_query_name.to_string()),
                                )?;
                                if let Some(val) = value {
                                       toql::tree::tree_predicate::TreePredicate::args(val , descendents, args)?
                                    }
                                }
                            )
                        },
                        JoinSelection::PreselectLeft => {
                            //  #[toql(preselect)] Option<T>
                            quote!(
                                #toql_query_name => {
                                    if let Some(val) = self. #field_name_ident .as_ref() {
                                            toql::tree::tree_predicate::TreePredicate::args(val   , descendents, args)?
                                        }
                                    }
                            )
                        },
                        selection @ JoinSelection::SelectInner |  selection @ JoinSelection::PreselectInner  => {
                            // Option<T>, T
                            let unwrap = if selection == &JoinSelection::SelectInner{
                                                quote!(.as_ref().ok_or(toql::error::ToqlError::ValueMissing(#field_name.to_string()))?)
                                            } else {
                                                quote!()
                                            };
                            let refer = if selection == &JoinSelection::PreselectInner {quote!(&) } else {quote!()};
                            quote!(
                                #toql_query_name => {
                                    toql::tree::tree_predicate::TreePredicate::args(#refer  self. #field_name_ident #unwrap,
                                    descendents, args)?
                                }
                            )
                        }

                    }
                );
                dispatch_predicate_columns_code.push(quote!(
                      #toql_query_name => {
                            <#field_base_type as toql::tree::tree_predicate::TreePredicate>::
                            columns(descendents)?
                        }
                ));
            }
            FieldKind::Merge(merge) => {
                dispatch_predicate_args_code.push(
                    if merge.selection == MergeSelection::Select {
                        quote!(
                            #toql_query_name => {
                                if let Some(ref fs) = self. #field_name_ident {
                                    for f in fs {
                                        <#field_base_type as toql::tree::tree_predicate::TreePredicate>::
                                        args(f, descendents.clone(), args)?
                                    }
                                }
                            }
                        )
                    } else {
                        quote!(
                            #toql_query_name => {
                                for f in & self. #field_name_ident {
                                    <#field_base_type as toql::tree::tree_predicate::TreePredicate>::
                                    args(f, descendents.clone(), args)?
                                }
                            }
                        )
                    }
                );
                dispatch_predicate_columns_code.push(quote!(
                    #toql_query_name => {
                        <#field_base_type as toql::tree::tree_predicate::TreePredicate>::
                        columns(descendents)?
                   }
                ));
            }
        }
    }
    // Generate token stream
    let mods = quote! {
            impl toql::tree::tree_predicate::TreePredicate for #struct_name_ident {
                 #[allow(unused_mut)]
                fn columns<'a, I>(mut descendents: I )
                    -> std::result::Result<Vec<String>, toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                    {
                        Ok(match descendents.next() {
                            Some(d) => match d.as_str() {
                                #(#dispatch_predicate_columns_code),*
                                f @ _ => {
                                        return Err(
                                            toql::error::ToqlError::SqlBuilderError (
                                             toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string())));
                                    }
                            },
                            None => {
                               <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            }
                        })
                    }
                #[allow(unused_mut)]
                fn args<'a, I>(
                    &self,
                    mut descendents: I,
                    args: &mut Vec<toql::sql_arg::SqlArg>
                ) -> std::result::Result<(), toql::error::ToqlError>
                where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                    match descendents.next() {
                        Some(d) => match d.as_str() {
                            #(#dispatch_predicate_args_code),*
                            f @ _ => {
                                    return Err(
                                        toql::error::ToqlError::SqlBuilderError (
                                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string())));
                                }
                        },
                        None => {
                                let key = <Self as toql::keyed::Keyed>::key(&self);
                            args.extend(<<Self as toql::keyed::Keyed>::Key as toql::key::Key>::params(&key));
                        }
                    }
                    Ok(())
                }
           }

            impl toql::tree::tree_predicate::TreePredicate for &#struct_name_ident {

                #[allow(unused_mut)]
                fn columns<'a, I>( mut descendents:  I )
                    -> std::result::Result<Vec<String>, toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                    {
                        <#struct_name_ident as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
                    }

                #[allow(unused_mut)]
                fn args<'a, I>(&self, mut descendents: I, args: &mut Vec<toql::sql_arg::SqlArg>)
                    -> std::result::Result<(), toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                    <#struct_name_ident as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
                }
            }
            impl toql::tree::tree_predicate::TreePredicate for &mut #struct_name_ident {
                #[allow(unused_mut)]
                fn columns<'a, I>( mut descendents: I )
                    -> std::result::Result<Vec<String>, toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                {
                    <#struct_name_ident as toql::tree::tree_predicate::TreePredicate>::columns( descendents)
                }

                 #[allow(unused_mut)]
                 fn args<'a, I>(&self, mut descendents: I, args: &mut Vec<toql::sql_arg::SqlArg>)
                    -> std::result::Result<(), toql::error::ToqlError>
                    where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
                {
                    <#struct_name_ident as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
                }
            }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
