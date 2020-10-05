
pub mod resolver_error;
pub mod resolver;

use crate::parameter::ParameterMap;
use crate::sql::Sql;
use crate::sql_arg::SqlArg;
use crate::{
    error::{Result, ToqlError},
    sql_builder::sql_builder_error::SqlBuilderError,
};

#[derive(Debug, Clone)]
pub enum SqlExprToken {
    
    SelfAlias,              
    OtherAlias,
    AuxParam(String),
    UnresolvedArg,

    Literal(String),
    Arg(SqlArg),
    Alias(String),
    InClause { column: String, args: Vec<SqlArg> },
}

#[derive(Debug)]
pub struct SqlExpr {
    tokens: Vec<SqlExprToken>,
}

impl SqlExpr {
    /*
    pub fn new() -> Self {
        SqlExpr {
            tokens:Vec::new()
        }
    } */

    pub fn from(tokens: Vec<SqlExprToken>) -> Self {
        SqlExpr { tokens }
    }
    pub fn new() -> Self {
        SqlExpr { tokens: Vec::new() }
    }

    pub fn push_literal(&mut self, lit: String) {
        self.tokens.push(SqlExprToken::Literal(lit));
    }
    pub fn push_self_alias(&mut self) {
        self.tokens.push(SqlExprToken::SelfAlias);
    }
    pub fn push_other_alias(&mut self) {
        self.tokens.push(SqlExprToken::OtherAlias);
    }
    pub fn push_arg(&mut self, arg: SqlArg) {
        self.tokens.push(SqlExprToken::Arg(arg));
    }
    pub fn is_empty(&mut self) -> bool {
        self.tokens.is_empty()
    }

    pub fn push_in_clause(&mut self, column: &str, arg: SqlArg) {
        if let Some(SqlExprToken::InClause {
            column: in_column,
            args,
        }) = self.tokens.last_mut()
        {
            if in_column == column {
                args.push(arg);
            } else {
                self.tokens.push(SqlExprToken::InClause {
                    column: column.to_string(),
                    args: vec![arg],
                });
            }
        } else {
            self.tokens.push(SqlExprToken::InClause {
                column: column.to_string(),
                args: vec![arg],
            });
        }
    }

    pub fn extend(&mut self, expr: SqlExpr) {
        self.tokens.extend(expr.tokens);
    }
    pub fn tokens(&self) -> &[SqlExprToken]{
        &self.tokens
    }

    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![
                SqlExprToken::SelfAlias,
                SqlExprToken::Literal(column_name),
            ],
        }
    }
}

    /* pub fn replace_self_alias(&mut self, canonical_alias: &str) {

        self.tokens.iter_mut().for_each(|&mut t|
        {
            if let SqlExprToken::SelfAlias() = t {
                t = SqlExprToken::CanonicalAlias(canonical_alias.to_string());
            }
        })

    }
    pub fn replace_aliases(&self, self_canonical_alias: &str, other_canonical_alias: Option<&str>) -> Self {

        let mut output_tokens = Vec::new();
        self.tokens.iter_mut().for_each(|t|
        {
            match  t {
                SqlExprToken::SelfAlias() => {
                     output_tokens.push(SqlExprToken::CanonicalAlias(self_canonical_alias.to_string()));
                }
                SqlExprToken::OtherAlias() if other_canonical_alias.is_some() => {
                     output_tokens.push(SqlExprToken::CanonicalAlias(other_canonical_alias.unwrap().to_string()));
                },
                tok @ _ => {
                    output_tokens.push(tok);
                }
            }
           
        });
        SqlExpr::from(output_tokens)

    } */

    // TODO make consuming self + args
    /* pub fn resolve(
        &self,
        self_alias: &str,
        other_alias: Option<&str>,
        aux_params: &ParameterMap,
        args: &[SqlArg],
    ) -> Result<Sql> {
        let mut iter = args.into_iter();
        let mut stmt = String::new();
        let mut output_args: Vec<SqlArg> = Vec::new();
        let mut aliased = false;

        
        for t in &self.tokens {
            match t {
                SqlExprToken::Literal(lit) => {
                    if aliased && !lit.starts_with(' ') {
                        stmt.push('.');
                        aliased = false;
                    }
                    stmt.push_str(&lit)
                }
                SqlExprToken::SelfAlias => {
                    stmt.push_str(self_alias);
                    aliased = true
                },
                 SqlExprToken::Alias(alias) => {
                    stmt.push_str(alias);
                    aliased = true
                },
                SqlExprToken::OtherAlias => {
                    stmt.push_str(other_alias.ok_or(ToqlError::ValueMissing("...".to_owned()))?);
                    aliased = true
                },
                 SqlExprToken::UnresolvedArg => {
                    stmt.push_str("?");
                   
                            let a = iter
                                .next()
                                .ok_or(ToqlError::ValueMissing("sql arg".to_string()))?;
                            output_args.push(a.to_owned());
                       
                    
                },
                SqlExprToken::Arg(arg) => {
                    stmt.push_str("?");
                    output_args.push(arg.to_owned());
                    
                }

                SqlExprToken::AuxParam(name) => {
                    stmt.push_str("?");
                    output_args.push(
                        aux_params
                            .get(&name)
                            .ok_or(ToqlError::ValueMissing(name.to_string()))?
                            .to_owned(),
                    );
                }
                SqlExprToken::InClause { column, args } => match args.len() {
                    0 => { /* Omit statement without any args */}
                    1 => {
                        if aliased && !column.starts_with(' ') {
                            stmt.push('.');
                            aliased = false;
                        }
                        stmt.push_str(column);
                        stmt.push_str(" =  ?");
                        output_args.push(args.get(0).unwrap().to_owned());
                    }
                    _ => {
                        if aliased && !column.starts_with(' ') {
                            stmt.push('.');
                            aliased = false;
                        }
                        stmt.push_str(" IN (");
                        for a in args {
                            stmt.push_str("?,");
                            output_args.push(a.to_owned());
                        }
                        stmt.pop();
                        stmt.push_str(")");
                    }
                },
            }
        }
        Ok(Sql(stmt, output_args))
    }
}
     */
