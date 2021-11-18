use std::collections::HashMap;
use syn::Path;

#[derive(Debug, PartialEq)]
pub enum SqlTarget {
    Column(String),
    Expression(String),
}

#[derive(Debug, PartialEq)]
pub enum RegularSelection {
    // Option<T>
    Select,
    // Option<Option<<T>>
    SelectNullable,
    // T
    Preselect,
    // #[toql(preselect)] Option<T>
    PreselectNullable,
}

#[derive(Debug)]
pub struct RegularField {
    pub sql_target: SqlTarget,
    pub key: bool,
    pub handler: Option<Path>,
    pub default_inverse_column: Option<String>,
    pub aux_params: HashMap<String, String>,
    pub foreign_key: bool, // Column of this field is used as foreign key
    pub selection: RegularSelection,
    pub skip_wildcard: bool,
}
