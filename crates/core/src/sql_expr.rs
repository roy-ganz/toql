

use crate::sql::Sql;
use crate::sql_arg::SqlArg;
use crate::parameter::ParameterMap;
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
/* 
    pub fn new() -> Self {
        SqlExpr {
            tokens:Vec::new()
        }
    } */

    pub fn from(tokens: Vec<SqlExprToken>) -> Self {
        SqlExpr {
            tokens
        }
    }
    pub fn new() -> Self {
        SqlExpr {
            tokens: Vec::new()
        }
    }

    pub fn push_literal(&mut self, lit: String) {
        self.tokens.push(SqlExprToken::Literal(lit));
    }
    pub fn push_self_alias(&mut self) {
        self.tokens.push(SqlExprToken::SelfAlias());
    }
    pub fn push_other_alias(&mut self) {
        self.tokens.push(SqlExprToken::OtherAlias());
    }
    pub fn is_empty(&mut self) -> bool{
        self.tokens.is_empty()
    }


   /*  pub fn extend(&mut self, expr: SqlExpr) {
            self.tokens.extend(self.tokens);
    } */


    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::SelfAlias(), SqlExprToken::Literal(column_name)]
        }
    }


    pub fn resolve(&self, self_alias: &str, other_alias: Option<&str>, aux_params: &ParameterMap) -> Result<Sql> {

        let mut stmt= String::new();
        let mut args :Vec<SqlArg> = Vec::new();
        let mut aliased = false;
        for t in &self.tokens {
            match t {
                SqlExprToken::Literal(lit) => { if aliased && !lit.starts_with(' ') { stmt.push('.'); aliased= false; } stmt.push_str(&lit)},
                SqlExprToken::SelfAlias() => {stmt.push_str(self_alias); aliased= true },
                SqlExprToken::OtherAlias() => {stmt.push_str(other_alias.ok_or(ToqlError::ValueMissing("...".to_owned()))?);aliased= true},
                SqlExprToken::AuxParam(name) => {
                    stmt.push_str("?"); 
                    args.push(aux_params.get(&name).ok_or(ToqlError::ValueMissing(name.to_string()))?.to_owned());
                    }
              }

        }
        Ok(Sql(stmt, args))
    }
}