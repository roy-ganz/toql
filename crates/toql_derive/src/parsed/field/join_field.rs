use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::Path;

#[derive(Debug, Clone)]
pub struct ColumnPair {
    pub this: String,
    pub other: String,
}

#[derive(Debug, PartialEq)]
pub enum JoinSelection {
    // Option<T>
    SelectInner,
    // Option<Option<<T>>
    SelectLeft,
    // T
    PreselectInner,
    // #[toql(preselect)] Option<T>
    PreselectLeft,
}
#[derive(Debug)]
pub struct JoinField {
    pub sql_join_table_name: String,
    pub join_alias: String,
    pub default_self_column_code: TokenStream,
    pub columns_map_code: TokenStream,
    pub translated_default_self_column_code: TokenStream,
    pub translated_columns_map_code: TokenStream,
    pub on_sql: Option<String>,
    pub key: bool,
    pub aux_params: HashMap<String, String>,
    pub columns: Vec<ColumnPair>,
    pub partial_table: bool,
    pub foreign_key: bool, // Column(s) of this join key is used as foreign key
    pub selection: JoinSelection,
    pub handler: Option<Path>,
}
