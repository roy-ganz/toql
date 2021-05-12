use crate::{sql_arg::SqlArg, sql_mapper_registry::SqlMapperRegistry, alias::AliasFormat};
use std::collections::{HashMap, HashSet};

pub struct Context {
    pub roles: HashSet<String>,
    pub aux_params: HashMap<String, SqlArg>,
    pub alias_format: AliasFormat,
}

impl Context {
    pub fn new(alias_format:AliasFormat) -> Self{
        Context {
           roles: HashSet::new(),
           aux_params: HashMap::new(),
           alias_format
       }
    }
}

impl Default for Context {
    fn default() -> Self {
       Context {
           roles: HashSet::new(),
           aux_params: HashMap::new(),
           alias_format: AliasFormat::Canonical
       }
    }
}