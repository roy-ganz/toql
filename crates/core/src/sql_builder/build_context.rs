use crate::sql_expr::SqlExpr;
use std::collections::HashSet;

pub(crate) struct BuildContext {
    pub(crate) query_home_path: String,

    pub(crate) joined_paths: HashSet<String>,

    pub(crate) selected_paths: HashSet<String>,
    pub(crate) selected_fields: HashSet<String>,
    pub(crate) all_fields_selected: bool,

    pub(crate) current_placeholder: u16,
  //  pub(crate) select_expr: SqlExpr,
   // pub(crate) selected_placeholders: HashSet<u16>,
}

impl BuildContext {
    pub fn new() -> Self {
        BuildContext {
            query_home_path: "".to_string(),
            joined_paths: HashSet::new(),
            selected_paths: HashSet::new(),
            selected_fields: HashSet::new(),
            all_fields_selected: true,
            current_placeholder: 0,
          //  select_expr: SqlExpr::new(),
           // selected_placeholders: HashSet::new(),
        }
    }
}
