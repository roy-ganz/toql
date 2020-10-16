
use proc_macro2::{Span};
use syn::Ident;

pub(crate) struct GeneratedMysqlMacros<'a> {
    rust_struct: &'a crate::sane::Struct,

}

impl<'a> GeneratedMysqlMacros<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlMacros {
        GeneratedMysqlMacros {
            rust_struct: &toql,
        }
    }
}

   

impl<'a> quote::ToTokens for GeneratedMysqlMacros<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
     
        let rust_struct_ident = &self.rust_struct.rust_struct_ident;
        let macro_name_index = Ident::new(&format!("toql_tree_index_{}", &rust_struct_ident), Span::call_site());
        let macro_name_merge = Ident::new(&format!("toql_tree_merge_{}", &rust_struct_ident), Span::call_site());

        let macro_name_key_from_row = Ident::new(&format!("toql_key_from_row_{}", &rust_struct_ident), Span::call_site());
        let macro_name_entity_from_row = Ident::new(&format!("toql_entity_from_row_{}", &rust_struct_ident), Span::call_site());


        let key = quote! {

            use  toql::mysql::mysql_row_try_get;
            
            #macro_name_index !(toql::mysql::mysql::Row, toql::mysql::error::ToqlMySqlError);
            #macro_name_merge !(toql::mysql::mysql::Row, toql::mysql::error::ToqlMySqlError);
            #macro_name_key_from_row !(toql::mysql::mysql::Row, mysql_row_try_get);
            #macro_name_entity_from_row !(toql::mysql::mysql::Row, mysql_row_try_get);


        };

        log::debug!(
            "Source code for `{}`:\n{}",
            rust_struct_ident,
            key.to_string()
        );
        tokens.extend(key);
    }
}
