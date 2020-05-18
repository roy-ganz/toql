

use crate::sql::{Sql, SqlArg};
use crate::parameter::Parameters;
use crate::error::{Result, ToqlError};

#[derive(Debug)]
pub enum SqlExprToken {
    Literal(String),
    SelfAlias(),
    OtherAlias(),
    AuxParam(String)
}

#[derive(Debug)]
pub struct SqlExpr {
    tokens: Vec<SqlExprToken>
}


impl SqlExpr {

    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::SelfAlias(), SqlExprToken::Literal(column_name)]
        }
    }


    pub fn resolve(&self, self_alias: &str, other_alias: Option<&str>, aux_params: &Parameters) -> Result<Sql> {

        let mut stmt= String::new();
        let mut args :Vec<SqlArg> = Vec::new();
        for t in &self.tokens {
            match t {
                SqlExprToken::Literal(lit) => stmt.push_str(&lit),
                SqlExprToken::SelfAlias() => stmt.push_str(self_alias),
                SqlExprToken::OtherAlias() => stmt.push_str(other_alias.ok_or(ToqlError::ValueMissing("...".to_owned()))?),
                SqlExprToken::AuxParam(name) => {
                    stmt.push_str("?");
                    args.push(aux_params.get(&name).ok_or(ToqlError::ValueMissing(name.to_string()))?.to_owned());
                    }
              }

        }
        Ok((stmt, args))
    }
}