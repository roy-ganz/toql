
use crate::alias_translator::AliasTranslator;
use std::collections::HashSet;
use super::sql_with_placeholders::SqlWithPlaceholders;

pub(crate) struct BuildContext {

    pub(crate) query_root_path: String,
    
    pub(crate) joined_paths: HashSet<String>,
  
    pub(crate) selected_paths: HashSet<String>,
    pub(crate) selected_fields: HashSet<String>,
    pub(crate )all_fields_selected: bool,

    pub(crate) current_placeholder: u16,
    pub(crate) select_sql: SqlWithPlaceholders,
    pub(crate) selected_placeholders: HashSet<u16>

}

impl BuildContext {
    pub fn new() -> Self {

        BuildContext {
            query_root_path: "".to_string(),
            joined_paths:HashSet::new(),
            selected_paths: HashSet::new(),
            selected_fields: HashSet::new(),
            all_fields_selected: true,
            current_placeholder: 0,
            select_sql: SqlWithPlaceholders::new(),
            selected_placeholders: HashSet::new()
        }

    }

   


}