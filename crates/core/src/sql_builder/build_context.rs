use crate::{query::field_path::FieldPath, sql_expr::SqlExpr};
use std::collections::HashSet;

pub(crate) struct BuildContext {
    pub(crate) query_home_path: String,
    pub(crate) local_joined_paths: HashSet<String>,
    pub(crate) local_selected_paths: HashSet<String>,
    pub(crate) local_selected_fields: HashSet<String>,
}

impl BuildContext {
    pub fn new() -> Self {
        BuildContext {
            query_home_path: "".to_string(),
            local_joined_paths: HashSet::new(),
            local_selected_paths: HashSet::new(),
            local_selected_fields: HashSet::new(),
        }
    }

    pub fn update_joins_from_selections(&mut self) {

        for path in self.local_selected_paths.iter().map(|p|FieldPath::from(p))
        {
            for p in path.step_down(){
                self.local_joined_paths.insert(p.to_string());
            }
        }
        for path in self.local_selected_fields.iter().map(|f| {let (_, p)= FieldPath::split_basename(f);p})
        {
            for p in path.step_down(){
                self.local_joined_paths.insert(p.to_string());
            }
        }

    }

}
