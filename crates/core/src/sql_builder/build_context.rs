
use crate::alias_translator::AliasTranslator;
use crate::query::field_path::FieldPath;
use std::collections::HashSet;
use super::sql_with_placeholders::SqlWithPlaceholders;

pub(crate) struct BuildContext {

    pub(crate) root_path: String,
    
    pub(crate) joined_paths: HashSet<String>,
    pub(crate) alias_translator : AliasTranslator,

    pub(crate) selected_paths: HashSet<String>,
    pub(crate) selected_fields: HashSet<String>,

    pub(crate) current_placeholder: u16,
    pub(crate) select_sql: SqlWithPlaceholders,
    pub(crate) selected_placeholders: HashSet<u16>

}

impl BuildContext {
    pub fn new(alias_translator: AliasTranslator) -> Self {

        BuildContext {
            root_path: "".to_string(),
            joined_paths:HashSet::new(),
            alias_translator,
            selected_paths: HashSet::new(),
            selected_fields: HashSet::new(),
            current_placeholder: 0,
            select_sql: SqlWithPlaceholders::new(),
            selected_placeholders: HashSet::new()
        }

    }


}