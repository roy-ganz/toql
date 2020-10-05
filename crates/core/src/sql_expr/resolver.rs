use super::resolver_error::ResolverError;
use crate::{
    alias::AliasFormat,
    alias_translator::AliasTranslator,
    parameter::ParameterMap,
    sql::Sql,
    sql_arg::SqlArg,
    sql_expr::{SqlExpr, SqlExprToken},
};
use std::{borrow::Cow, collections::HashSet};

pub struct Resolver<'a> {
    self_alias: Option<&'a str>,
    other_alias: Option<&'a str>,
    arguments: Option<&'a [SqlArg]>,
    aux_params: Option<&'a ParameterMap<'a>>,
    placeholders: Option<&'a HashSet<u16>>,
    // alias_translator: Option<&'a AliasTranslator>
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            self_alias: None,
            other_alias: None,
            arguments: None,
            aux_params: None,
            placeholders: None,
            // alias_translator: None
        }
    }

    pub fn with_self_alias(mut self, alias: &'a str) -> Self {
        self.self_alias = Some(alias);
        self
    }
    pub fn with_other_alias(mut self, alias: &'a str) -> Self {
        self.other_alias = Some(alias);
        self
    }
    pub fn with_aux_params(mut self, aux_params: &'a ParameterMap<'a>) -> Self {
        self.aux_params = Some(aux_params);
        self
    }
    pub fn with_arguments(mut self, arguments: &'a [SqlArg]) -> Self {
        self.arguments = Some(arguments);
        self
    }
    /*   pub fn with_alias_translator(mut self, alias_translator: &'a AliasTranslator) ->  Self{
        self.alias_translator= Some(alias_translator);
        self
    } */

    pub fn resolve(&self, sql_expr: &'a SqlExpr) -> std::result::Result<SqlExpr, ResolverError> {
        let mut tokens = Vec::new();
        let mut arg_iter = if self.arguments.is_some() {
            Some(self.arguments.unwrap())
        } else {
            None
        };

        for token in sql_expr.tokens() {
            tokens.push(self.resolve_token(token)?.into_owned())
        }

        Ok(SqlExpr::from(tokens))
    }

    pub fn to_sql(
        &self,
        sql_expr: &SqlExpr,
        alias_translator: &mut AliasTranslator,
    ) -> Result<Sql, ResolverError> {
        let mut stmt = String::new();
        let mut args: Vec<SqlArg> = Vec::new();

        for unresolved_token in &sql_expr.tokens {
            let mut token = self.resolve_token(unresolved_token)?;
            Self::token_to_sql(token.to_mut(), alias_translator, &mut stmt, &mut args)?;
        }

        Ok(Sql(stmt, args))
    }

    fn resolve_token(
        &self,
        token: &'a SqlExprToken,
    ) -> Result<Cow<'a, SqlExprToken>, ResolverError> {
        let arg_iter = if self.arguments.is_some() {
            Some(self.arguments.unwrap())
        } else {
            None
        };

        match token {
            SqlExprToken::SelfAlias if self.self_alias.is_some() => Ok(Cow::Owned(
                SqlExprToken::Alias(self.self_alias.unwrap().to_string()),
            )),
            SqlExprToken::OtherAlias if self.other_alias.is_some() => Ok(Cow::Owned(
                SqlExprToken::Alias(self.other_alias.unwrap().to_string()),
            )),
            SqlExprToken::AuxParam(name) if self.aux_params.is_some() => {
                let arg = self
                    .aux_params
                    .unwrap()
                    .get(&name)
                    .ok_or(ResolverError::AuxParamMissing(name.to_string()))?
                    .to_owned();
                Ok(Cow::Owned(SqlExprToken::Arg(arg)))
            }
            SqlExprToken::UnresolvedArg if arg_iter.is_some() => {
                let arg = arg_iter
                    .unwrap()
                    .iter()
                    .next()
                    .ok_or(ResolverError::ArgumentMissing)?;

                Ok(Cow::Owned(SqlExprToken::Arg(arg.to_owned())))
            }
            tok @ _ => Ok(Cow::Borrowed(tok)),
        }
    }

    pub fn token_to_sql(
        token: &SqlExprToken,
        alias_translator: &mut AliasTranslator,
        stmt: &mut String,
        args: &mut Vec<SqlArg>,
    ) -> std::result::Result<(), ResolverError> {
        let mut aliased = false;

        match token {
            SqlExprToken::SelfAlias => return Err(ResolverError::UnresolvedSelfAlias),
            SqlExprToken::OtherAlias => return Err(ResolverError::UnresolvedOtherAlias),
            SqlExprToken::UnresolvedArg => return Err(ResolverError::UnresolvedArgument),
            SqlExprToken::AuxParam(name) => {
                return Err(ResolverError::UnresolvedAuxParameter(name.to_owned()))
            }

            SqlExprToken::Placeholder(number, expr) => todo!(),

            SqlExprToken::Literal(lit) => {
                if aliased && !lit.starts_with(' ') {
                    stmt.push('.');
                    aliased = false;
                }
                stmt.push_str(&lit)
            }

            SqlExprToken::Alias(canonical_alias) => {
                let alias = alias_translator.translate(canonical_alias);
                stmt.push_str(&alias);
                aliased = true
            }

            SqlExprToken::Arg(arg) => {
                stmt.push_str("?");
                args.push(arg.to_owned());
            }

            SqlExprToken::InClause { column, args: ars } => match args.len() {
                0 => { /* Omit statement without any args */ }
                1 => {
                    if aliased && !column.starts_with(' ') {
                        stmt.push('.');
                        aliased = false;
                    }
                    stmt.push_str(column);
                    stmt.push_str(" =  ?");
                    args.push(ars.get(0).unwrap().to_owned());
                }
                _ => {
                    if aliased && !column.starts_with(' ') {
                        stmt.push('.');
                        aliased = false;
                    }
                    stmt.push_str(" IN (");
                    for a in ars {
                        stmt.push_str("?,");
                        args.push(a.to_owned());
                    }
                    stmt.pop();
                    stmt.push_str(")");
                }
            },
        }
        Ok(())
    }
}
