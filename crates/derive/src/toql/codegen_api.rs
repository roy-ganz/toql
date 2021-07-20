use syn::Ident;

pub(crate) struct CodegenApi<'a> {
    struct_ident: &'a Ident,
}

impl<'a> CodegenApi<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenApi {
        CodegenApi {
            struct_ident: &toql.rust_struct_ident,
        }
    }
}

impl<'a> quote::ToTokens for CodegenApi<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // let vis = &self.rust_struct.rust_struct_visibility;

        let struct_ident = self.struct_ident;
        let api = quote! {


            impl<R, E> toql::toql_api::Load<R,E> for #struct_ident
            where
                    Self: toql::keyed::Keyed
                    + toql::sql_mapper::mapped::Mapped
                    + toql::tree::tree_map::TreeMap
                    + toql::from_row::FromRow<R,E>
                    + toql::tree::tree_predicate::TreePredicate
                    + toql::tree::tree_index::TreeIndex<R, E>
                    + toql::tree::tree_merge::TreeMerge<R, E> + std::fmt::Debug,
                    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R,E>, E: std::convert::From<toql::error::ToqlError>
            {}

            impl<R, E> toql::toql_api::Load<R,E> for & #struct_ident
            where
                    Self: toql::keyed::Keyed
                    + toql::sql_mapper::mapped::Mapped
                    + toql::tree::tree_map::TreeMap
                    + toql::from_row::FromRow<R,E>
                    + toql::tree::tree_predicate::TreePredicate
                    + toql::tree::tree_index::TreeIndex<R, E>
                    + toql::tree::tree_merge::TreeMerge<R, E> + std::fmt::Debug,
                    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R,E>, E: std::convert::From<toql::error::ToqlError>
            {}

            impl toql::toql_api::Insert for #struct_ident
            where
                Self: toql::tree::tree_insert::TreeInsert
                + toql::sql_mapper::mapped::Mapped
                + toql::tree::tree_map::TreeMap
                + toql::tree::tree_identity::TreeIdentity {}

            impl toql::toql_api::Insert for &mut #struct_ident
            where
                Self: toql::tree::tree_insert::TreeInsert
                + toql::sql_mapper::mapped::Mapped
                + toql::tree::tree_map::TreeMap
                + toql::tree::tree_identity::TreeIdentity {}

            impl toql::toql_api::Update for #struct_ident
            where
            Self: toql::tree::tree_update::TreeUpdate
            + toql::sql_mapper::mapped::Mapped
            + toql::tree::tree_map::TreeMap
            + toql::tree::tree_identity::TreeIdentity
            + toql::tree::tree_predicate::TreePredicate
            + toql::tree::tree_insert::TreeInsert {}

            impl toql::toql_api::Update for &mut #struct_ident
            where
            Self:  toql::tree::tree_update::TreeUpdate
            + toql::sql_mapper::mapped::Mapped
            + toql::tree::tree_map::TreeMap
            + toql::tree::tree_identity::TreeIdentity
            + toql::tree::tree_predicate::TreePredicate
            + toql::tree::tree_insert::TreeInsert {}

            impl toql::toql_api::Count  for #struct_ident
            where
            Self: toql::keyed::Keyed
            +  toql::sql_mapper::mapped::Mapped
            + std::fmt::Debug {}

            impl toql::toql_api::Count  for &#struct_ident
            where
            Self: toql::keyed::Keyed
            + toql::sql_mapper::mapped::Mapped
            + std::fmt::Debug {}

            impl toql::toql_api::Delete  for #struct_ident
            where
            Self: toql::sql_mapper::mapped::Mapped
            + toql::tree::tree_map::TreeMap
            + std::fmt::Debug {}

            impl toql::toql_api::Delete  for &#struct_ident
            where
            Self:  toql::sql_mapper::mapped::Mapped
            + toql::tree::tree_map::TreeMap
            + std::fmt::Debug {}

        };

        log::debug!("Source code for `{}`:\n{}", struct_ident, api.to_string());
        tokens.extend(api);
    }
}
