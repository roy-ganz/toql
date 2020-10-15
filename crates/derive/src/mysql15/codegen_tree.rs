use crate::sane::FieldKind;
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) struct GeneratedMysqlTree<'a> {
    rust_struct: &'a crate::sane::Struct,

}

impl<'a> GeneratedMysqlTree<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlTree {
        GeneratedMysqlTree {
            rust_struct: &toql,
        }
    }
}

   

impl<'a> quote::ToTokens for GeneratedMysqlTree<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
     
        let rust_struct_ident = &self.rust_struct.rust_struct_ident;
        let macro_name_index = Ident::new(&format!("toql_tree_index_{}", &rust_struct_ident), Span::call_site());
        let macro_name_merge = Ident::new(&format!("toql_tree_merge_{}", &rust_struct_ident), Span::call_site());


        let key = quote! {


            #macro_name_index !(toql::mysql::mysql::Row, toql::mysql::error::ToqlMySqlError);
            #macro_name_merge !(toql::mysql::mysql::Row, toql::mysql::error::ToqlMySqlError);



        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_struct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
