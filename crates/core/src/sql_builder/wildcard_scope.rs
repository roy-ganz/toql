use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum WildcardScope {
    All,
    Only(HashSet<String>),
}

impl WildcardScope {
    pub fn contains_field(&self, field: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => scopes.contains(field),
        }
    }
    pub fn contains_all_fields_from_path(&self, path: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                let mut field = String::from(path);
                if !path.is_empty() && !path.ends_with('_') {
                    field.push('_');
                }
                field.push('*');
                scopes.contains(field.as_str())
            }
        }
    }
    pub fn contains(&self, field_with_path: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                scopes.contains(field_with_path)
                    || if !field_with_path.ends_with('*') {
                        // If field is provided check for all fields
                        let mut path = field_with_path.trim_end_matches(|c| c != '_').to_string();
                        path.push('*');
                        scopes.contains(path.as_str())
                    } else {
                        false
                    }
            }
        }
    }
    pub fn contains_path(&self, path: &str) -> bool {
        let path = path.trim_end_matches('_'); // Remove optional trailing underscore
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                let mut wildcard_path = path.to_owned();
                wildcard_path.push_str("_*");
                // Check if path with wildcard exists or any field with that path
                scopes.contains(wildcard_path.as_str())
                    || scopes
                        .iter()
                        .any(|s| s.trim_end_matches(|c| c != '_').trim_end_matches('_') == path)
            }
        }
    }
}
