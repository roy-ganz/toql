use super::context::Context;
use crate::{alias_format::AliasFormat, sql_arg::SqlArg};
use std::collections::{HashMap, HashSet};

pub struct ContextBuilder {
    pub roles: HashSet<String>,
    pub aux_params: HashMap<String, SqlArg>,
    pub alias_format: AliasFormat,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            alias_format: AliasFormat::Canonical,
        }
    }

    pub fn with_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }
    pub fn with_aux_params(mut self, aux_params: HashMap<String, SqlArg>) -> Self {
        self.aux_params = aux_params;
        self
    }
    pub fn with_alias_format(mut self, alias_format: AliasFormat) -> Self {
        self.alias_format = alias_format;
        self
    }
    pub fn build(self) -> Context {
        Context {
            roles: self.roles,
            aux_params: self.aux_params,
            alias_format: self.alias_format,
        }
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
