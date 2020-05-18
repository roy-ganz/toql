use crate::sql::SqlArg;
use std::collections::HashMap;

pub struct Parameters<'a>(&'a [&'a HashMap<String, SqlArg>]);

impl<'a> Parameters<'a> {

    pub fn new(params: &'a [&'a HashMap<String, SqlArg>]) -> Self {
        Parameters (params)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|m|m.contains_key(name))
    } 

     pub fn get(&self, name: &str) -> Option<&'a SqlArg> {
        for p in self.0 {
            match p.get(name) {
                Some(p) => return Some(p),
                None => {}
            }
        }
        None
    } 
}




