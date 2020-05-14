
use std::collections::HashSet;


pub struct BuildContext {


    pub root_path: String,
    
    pub selected_paths: HashSet<String>

}

impl BuildContext {
    pub fn new() -> Self {

        BuildContext {
            root_path: "".to_string(),
            selected_paths:HashSet::new()
        }

    }

}