use syn::Ident;

pub(crate) struct CodegenApi<'a> {
    struct_ident: &'a Ident,
    skip_mut: bool,
}

impl<'a> CodegenApi<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenApi {
        CodegenApi {
            struct_ident: &toql.rust_struct_ident,
            skip_mut: toql.skip_mut,
        }
    }
}

impl<'a> quote::ToTokens for CodegenApi<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // let vis = &self.rust_struct.rust_struct_visibility;

        let struct_ident = self.struct_ident;
        let mut_api = if !self.skip_mut {
            quote!(
            impl toql::toql_api::insert::Insert for #struct_ident
                      where
                          Self: toql::tree::tree_insert::TreeInsert
                          + toql::table_mapper::mapped::Mapped
                          + toql::tree::tree_map::TreeMap
                          + toql::tree::tree_identity::TreeIdentity {}

                      impl toql::toql_api::insert::Insert for &mut #struct_ident
                      where
                          Self: toql::tree::tree_insert::TreeInsert
                          + toql::table_mapper::mapped::Mapped
                          + toql::tree::tree_map::TreeMap
                          + toql::tree::tree_identity::TreeIdentity {}

                      impl toql::toql_api::update::Update for #struct_ident
                      where
                      Self: toql::tree::tree_update::TreeUpdate
                      + toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                      + toql::tree::tree_identity::TreeIdentity
                      + toql::tree::tree_predicate::TreePredicate
                      + toql::tree::tree_insert::TreeInsert {}

                      impl toql::toql_api::update::Update for &mut #struct_ident
                      where
                      Self:  toql::tree::tree_update::TreeUpdate
                      + toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                      + toql::tree::tree_identity::TreeIdentity
                      + toql::tree::tree_predicate::TreePredicate
                      + toql::tree::tree_insert::TreeInsert {}


                       impl toql::toql_api::delete::Delete  for #struct_ident
                      where
                      Self: toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                       {}

                      impl toql::toql_api::delete::Delete  for &#struct_ident
                      where
                      Self:  toql::table_mapper::mapped::Mapped
                      + toql::tree::tree_map::TreeMap
                      {}
                      )
        } else {
            quote!()
        };

        let api = quote! {


            impl<R, E> toql::toql_api::load::Load<R,E> for #struct_ident
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

            impl<R, E> toql::toql_api::load::Load<R,E> for & #struct_ident
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



            impl toql::toql_api::count::Count  for #struct_ident
            where
            Self: toql::keyed::Keyed
            +  toql::table_mapper::mapped::Mapped
             {}

            impl toql::toql_api::count::Count  for &#struct_ident
            where
            Self: toql::keyed::Keyed
            + toql::table_mapper::mapped::Mapped
            {}

        };

        log::debug!(
            "Source code for `{}`:\n{}\n{}",
            struct_ident,
            api.to_string(),
            mut_api.to_string()
        );
        tokens.extend(mut_api);
        tokens.extend(api);
    }
}
