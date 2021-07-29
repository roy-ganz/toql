

/// Manages role restrictions for fields
pub(crate) struct Restrictor  {
      
    /// Restriction tree
    /// Children inherit role restrictions form parent.
    /// Eg role 'admin' on  path 'user' will also be on field 'user_address'.
    qualified_fields: HashMap<String, Vec<String>>;

    /// Fully nested path tree
    /// If the tree does not contain an immediate parent path
    /// the path is added to pending paths.
    /// Paths also inherit role restrictions form parent paths.
    paths: HashMap<String, Vec<String>>; 

    /// Paths that do not contain an immediate parent path in tree
    /// Their required roles may not be complete as an immediate parent path
    /// that is inserted after them may impose additional role restrictions.
    /// After each path insertion, the orpahned paths are checked to be included 
    /// into the path tree.
    /// In a good mapping sequence no orphaned paths should occur.
    orphaned_paths: HashMap<String, Vec<String>>; 


    /// Validation cache to speed up validation. 
    /// Contains last validation result
    last_path_validation: Option<(&str, bool)>; 
}


impl Restrictor {
    pub fn insert_field(qualified_field: &str, required_roles: Hashset<String>) {

    }

    /// Insert restricted path. 
    /// Restrictions will be inherited from parent path(s) 
    pub fn insert_path(path: &str, required_roles: Hashset<String>) {

    }

    /// Checks if a qualified field and its path contain the asserted roles.
    /// If no roles are registered for a qualified field, this returns true
    pub fn verify_field(qualified_field: &str, asserted_roles: Hashset<String>) -> bool {
        
        true
    }
    
}
