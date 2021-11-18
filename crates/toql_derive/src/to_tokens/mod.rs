pub(crate) mod api;
pub(crate) mod entity_from_row;
pub(crate) mod key;
pub(crate) mod key_from_row;
pub(crate) mod mapped;
pub(crate) mod query_fields;
pub(crate) mod tree_identity;
pub(crate) mod tree_index;
pub(crate) mod tree_insert;
pub(crate) mod tree_map;
pub(crate) mod tree_merge;
pub(crate) mod tree_predicate;
pub(crate) mod tree_update;

use crate::parsed::parsed_struct::ParsedStruct;
use proc_macro2::TokenStream;

impl quote::ToTokens for ParsedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        api::to_tokens(self, tokens);
        key::to_tokens(self, tokens);
        mapped::to_tokens(self, tokens);
        key_from_row::to_tokens(self, tokens);
        entity_from_row::to_tokens(self, tokens);
        query_fields::to_tokens(self, tokens);
        tree_insert::to_tokens(self, tokens);
        tree_update::to_tokens(self, tokens);
        tree_index::to_tokens(self, tokens);
        tree_identity::to_tokens(self, tokens);
        tree_map::to_tokens(self, tokens);
        tree_predicate::to_tokens(self, tokens);
        tree_merge::to_tokens(self, tokens);
    }
}
