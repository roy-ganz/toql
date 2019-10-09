use heck::CamelCase;
use heck::MixedCase;
use heck::ShoutySnakeCase;
use heck::SnakeCase;

use crate::annot::RenameCase;

pub(crate) fn rename(string: &str, renaming: &Option<RenameCase>) -> String {
    match renaming {
        Some(RenameCase::CamelCase) => string.to_camel_case(),
        Some(RenameCase::SnakeCase) => string.to_snake_case(),
        Some(RenameCase::ShoutySnakeCase) => string.to_shouty_snake_case(),
        Some(RenameCase::MixedCase) => string.to_mixed_case(),
        None => string.to_owned(),
    }
}


pub(crate) struct JoinInfo {

    pub(crate) joined_table: String,
    pub(crate) on_expr : String,
    pub (crate) key_names: Vec<String>,
    pub (crate) key_types: Vec<String>
}

pub(crate) fn join_info() -> JoinInfo {

    JoinInfo {
        joined_table: String::from(""),
        on_expr: String::from(""),
        key_names: Vec::new(),
        key_types: Vec::new()

    }
}
/* 
pub(crate) enum FieldType {
    Regular,
    Join,   // JOin(JoinInfo)
    Merge
}

pub(crate) fn field_type() ->FieldType {
    FieldType::Regular

}
 */