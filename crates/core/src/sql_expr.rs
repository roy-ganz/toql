use crate::parameter::ParameterMap;
use crate::sql::Sql;
use crate::sql_arg::SqlArg;
use crate::{
    error::{Result, ToqlError},
    sql_builder::sql_builder_error::SqlBuilderError,
};

#[derive(Debug)]
pub enum SqlExprArg {
    Unresolved,
    Resolved(SqlArg),
}

#[derive(Debug)]
pub enum SqlExprToken {
    Literal(String),
    SelfAlias(),
    OtherAlias(),
    AuxParam(String),
    Arg(SqlExprArg),
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
        self.tokens.push(SqlExprToken::SelfAlias());
    }
    pub fn push_other_alias(&mut self) {
        self.tokens.push(SqlExprToken::OtherAlias());
    }
    pub fn push_arg(&mut self, arg: SqlExprArg) {
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

    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![
                SqlExprToken::SelfAlias(),
                SqlExprToken::Literal(column_name),
            ],
        }
    }

    // TODO make consuming self + args
    pub fn resolve(
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
                SqlExprToken::SelfAlias() => {
                    stmt.push_str(self_alias);
                    aliased = true
                }
                SqlExprToken::OtherAlias() => {
                    stmt.push_str(other_alias.ok_or(ToqlError::ValueMissing("...".to_owned()))?);
                    aliased = true
                }
                SqlExprToken::Arg(arg) => {
                    stmt.push_str("?");
                    match arg {
                        SqlExprArg::Unresolved => {
                            let a = iter
                                .next()
                                .ok_or(ToqlError::ValueMissing("sql arg".to_string()))?;
                            output_args.push(a.to_owned());
                        }
                        SqlExprArg::Resolved(a) => output_args.push(a.to_owned()),
                    }
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
