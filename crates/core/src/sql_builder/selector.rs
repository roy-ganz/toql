
 


/// Keeps field and path selections
/// Selecting a field will also select all parent paths
pub(crate) struct Selector  {
    /// Explicit selected fields
    qualified_fields: HashSet<String>; 
    /// Paths with all fields selected
    paths: HashSet<String>; 
}

impl Selector {
   
    pub fn select_field(&self, field: &Field) {

        self.qualified_fields.insert(field.qualified_name());
        
    }
    pub fn select_all_of_path(&self, path: &FieldPath) {
        for parent in path.parents() {
            if self.paths.contains(parent) { break;}
            paths.insert(parent);
        }
    }

    // Returns true, if field is selected
    pub fn contains_field(&self, field: &Field) -> bool {
        // Qualified name is name with path
        self.qualified_fields.contains(&field.qualified_name()) || self.path.contains(field.path())
    }
   
}






