use crate::{alias_format::AliasFormat, sql_arg::SqlArg};
use std::collections::{HashMap, HashSet};

pub struct Context {
    pub roles: HashSet<String>,
    pub aux_params: HashMap<String, SqlArg>,
    pub alias_format: AliasFormat,
}

impl Context {
    pub fn new(alias_format: AliasFormat) -> Self {
        Context {
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            alias_format,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(AliasFormat::Canonical)
    }
}
