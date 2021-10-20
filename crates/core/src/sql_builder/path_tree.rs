//! Build join dependencies

use crate::query::field_path::FieldPath;
use std::collections::HashMap;
use std::collections::HashSet;

/// Holds different roots and nodes
/// Its used to build the join dependencies to
/// do proper nested joins with parens.
///
/// The tree build up like this:
///
/// [user] = [user_address, user_folder]
/// [user_folder] = [ user_folder_owner]
/// [user_folder_owner] =[]
/// [user address] =[]

#[derive(Debug)]
pub struct PathTree {
    roots: HashSet<String>,
    nodes: HashMap<String, HashSet<String>>,
}

impl PathTree {
    /// Create new tree.
    pub fn new() -> Self {
        PathTree {
            roots: HashSet::new(),
            nodes: HashMap::new(),
        }
    }
    /// Get all roots.
    pub fn roots(&self) -> &HashSet<String> {
        &self.roots
    }
    /// Get all nodes for a head.
    pub fn nodes(&self, name: &str) -> Option<&HashSet<String>> {
        self.nodes.get(name)
    }

    /// Insert a [FieldPath] in the tree.
    /// This will walk down the paths and insert the parts
    /// as roots, heads and nodes.
    pub fn insert(&mut self, path: &FieldPath) {
        let mut parents = path.ancestors().skip(1);

        for a in path.ancestors() {
            // If Parent exists, its not a tree root

            match parents.next() {
                Some(p) => {
                    // If parent is already in the tree, Add child and leave inner for loop
                    let j = self
                        .nodes
                        .entry(p.as_str().to_string())
                        .or_insert_with(HashSet::new);
                    j.insert(a.as_str().to_string());

                    if self.nodes.contains_key(a.as_str()) {
                        break;
                    }
                }
                None => {
                    self.roots.insert(a.as_str().to_string());
                    break;
                }
            }
        }
    }
}
