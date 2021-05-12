// Path tree
// Eg [user] = [user_address, user_folder]
// [user_folder] = [ user_folder_owner]
// [user_folder_owner] =[]
// [user address] =[]

use crate::query::field_path::FieldPath;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct PathTree {
    roots: HashSet<String>,
    nodes: HashMap<String, HashSet<String>>,
}

impl PathTree {
    pub fn new() -> Self {
        PathTree {
            roots: HashSet::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn roots(&self) -> &HashSet<String> {
        &self.roots
    }
    pub fn nodes(&self, name: &str) -> Option<&HashSet<String>> {
        self.nodes.get(name)
    }

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
                        .or_insert(HashSet::new());
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
