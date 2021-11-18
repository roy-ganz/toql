use crate::parsed::parsed_struct::ParsedStruct;
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let struct_name_ident = &parsed_struct.struct_name;

    let api = quote! {

         impl toql::toql_api::insert::Insert for #struct_name_ident
                  where
                      Self: toql::tree::tree_insert::TreeInsert
                      + toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                      + toql::tree::tree_identity::TreeIdentity {}

                  impl toql::toql_api::insert::Insert for &mut #struct_name_ident
                  where
                      Self: toql::tree::tree_insert::TreeInsert
                      + toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                      + toql::tree::tree_identity::TreeIdentity {}

                  impl toql::toql_api::update::Update for #struct_name_ident
                  where
                  Self: toql::tree::tree_update::TreeUpdate
                  + toql::table_mapper::mapped::Mapped
                  + toql::tree::tree_map::TreeMap
                  + toql::tree::tree_identity::TreeIdentity
                  + toql::tree::tree_predicate::TreePredicate
                  + toql::tree::tree_insert::TreeInsert {}

                  impl toql::toql_api::update::Update for &mut #struct_name_ident
                  where
                  Self:  toql::tree::tree_update::TreeUpdate
                  + toql::table_mapper::mapped::Mapped
                  + toql::tree::tree_map::TreeMap
                  + toql::tree::tree_identity::TreeIdentity
                  + toql::tree::tree_predicate::TreePredicate
                  + toql::tree::tree_insert::TreeInsert {}


                   impl toql::toql_api::delete::Delete  for #struct_name_ident
                  where
                  Self: toql::table_mapper::mapped::Mapped
                  + toql::tree::tree_map::TreeMap
                   {}

                  impl toql::toql_api::delete::Delete  for &#struct_name_ident
                  where
                  Self:  toql::table_mapper::mapped::Mapped
                  + toql::tree::tree_map::TreeMap
                  {}



        impl<R, E> toql::toql_api::load::Load<R,E> for #struct_name_ident
        where
                Self: toql::keyed::Keyed
                + toql::table_mapper::mapped::Mapped
                + toql::tree::tree_map::TreeMap
                + toql::from_row::FromRow<R,E>
                + toql::tree::tree_predicate::TreePredicate
                + toql::tree::tree_index::TreeIndex<R, E>
                + toql::tree::tree_merge::TreeMerge<R, E>,
                <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R,E>, E: std::convert::From<toql::error::ToqlError>
        {}

        impl<R, E> toql::toql_api::load::Load<R,E> for & #struct_name_ident
        where
                Self: toql::keyed::Keyed
                + toql::table_mapper::mapped::Mapped
                + toql::tree::tree_map::TreeMap
                + toql::from_row::FromRow<R,E>
                + toql::tree::tree_predicate::TreePredicate
                + toql::tree::tree_index::TreeIndex<R, E>
                + toql::tree::tree_merge::TreeMerge<R, E>,
                <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R,E>, E: std::convert::From<toql::error::ToqlError>
        {}



        impl toql::toql_api::count::Count  for #struct_name_ident
        where
        Self: toql::keyed::Keyed
        +  toql::table_mapper::mapped::Mapped
         {}

        impl toql::toql_api::count::Count  for &#struct_name_ident
        where
        Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        {}

    };

    log::debug!(
        "Source code for `{}`:\n{}",
        struct_name_ident,
        api.to_string(),
    );

    tokens.extend(api);
}
