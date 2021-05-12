use super::{resolver_error::ResolverError, PredicateColumn};
use crate::{
    alias_translator::AliasTranslator,
    parameter_map::ParameterMap,
    sql::Sql,
    sql_arg::SqlArg,
    sql_expr::{SqlExpr, SqlExprToken},
};
use std::{borrow::Cow, collections::HashMap};

pub struct Resolver<'a> {
    self_alias: Option<&'a str>,
    other_alias: Option<&'a str>,
    arguments: Option<&'a [SqlArg]>,
    aux_params: Option<&'a ParameterMap<'a>>,
    // alias_translator: Option<&'a AliasTranslator>
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            self_alias: None,
            other_alias: None,
            arguments: None,
            aux_params: None,
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

    pub fn replace_aux_params(
        sql_expr: SqlExpr,
        aux_params_exprs: &HashMap<String, SqlExpr>,
    ) -> SqlExpr {
        let mut tokens = Vec::new();

        for token in sql_expr.tokens {
            if let SqlExprToken::AuxParam(ref name) = token {
                if let Some(expr) = aux_params_exprs.get(name) {
                    tokens.extend_from_slice(&expr.tokens);
                }
            } else {
                tokens.push(token);
            }
        }
        SqlExpr::from(tokens)
    }

    pub fn resolve(&self, sql_expr: &'a SqlExpr) -> std::result::Result<SqlExpr, ResolverError> {
        let mut tokens = Vec::new();

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
                    .ok_or_else(|| ResolverError::AuxParamMissing(name.to_string()))?
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
            SqlExprToken::Predicate { columns, args }
                if self.self_alias.is_some() || self.other_alias.is_some() =>
            {
                // TODO optimise so that arguments are not copied
                // maybe take self instead of &self

                let mut changed_columns: Vec<PredicateColumn> = Vec::new();
                let mut changed = false;

                for c in columns {
                    changed_columns.push(match c {
                        PredicateColumn::SelfAliased(a) => {
                            changed = true;
                            if self.self_alias.is_some() {
                                PredicateColumn::Aliased(
                                    self.self_alias.unwrap().to_owned(),
                                    a.to_owned(),
                                )
                            } else {
                                PredicateColumn::SelfAliased(a.to_owned())
                            }
                        }
                        PredicateColumn::OtherAliased(a) => {
                            changed = true;
                            if self.other_alias.is_some() {
                                PredicateColumn::Aliased(
                                    self.other_alias.unwrap().to_owned(),
                                    a.to_owned(),
                                )
                            } else {
                                PredicateColumn::OtherAliased(a.to_owned())
                            }
                        }
                        PredicateColumn::Literal(l) => PredicateColumn::Literal(l.to_owned()),
                        PredicateColumn::Aliased(a, c) => {
                            PredicateColumn::Aliased(a.to_owned(), c.to_owned())
                        }
                    });
                }

                if changed {
                    Ok(Cow::Owned(SqlExprToken::Predicate {
                        columns: changed_columns,
                        args: args.to_owned(),
                    }))
                } else {
                    // Pattern bindings are unstable, so we can't return Cow::Borrowed(token)
                    Ok(Cow::Owned(SqlExprToken::Predicate {
                        columns: columns.to_owned(),
                        args: args.to_owned(),
                    }))
                }
            }
            tok => Ok(Cow::Borrowed(tok)),
        }
    }

    pub fn token_to_sql(
        token: &SqlExprToken,
        alias_translator: &mut AliasTranslator,
        stmt: &mut String,
        args: &mut Vec<SqlArg>,
    ) -> std::result::Result<(), ResolverError> {
        match token {
            SqlExprToken::SelfAlias => return Err(ResolverError::UnresolvedSelfAlias),
            SqlExprToken::OtherAlias => return Err(ResolverError::UnresolvedOtherAlias),
            SqlExprToken::UnresolvedArg => return Err(ResolverError::UnresolvedArgument),
            SqlExprToken::AuxParam(name) => {
                return Err(ResolverError::UnresolvedAuxParameter(name.to_owned()))
            }

            SqlExprToken::Literal(lit) => stmt.push_str(&lit),

            SqlExprToken::Alias(canonical_alias) => {
                let alias = alias_translator.translate(canonical_alias);
                stmt.push_str(&alias);
            }

            SqlExprToken::Arg(arg) => {
                stmt.push('?');
                args.push(arg.to_owned());
            }

            SqlExprToken::Predicate { columns, args: a } => match columns.len() {
                0 => { /* Omit statement if no columns are provied */ }
                1 => {
                    match columns.get(0).unwrap() {
                        PredicateColumn::SelfAliased(_) => {
                            return Err(ResolverError::UnresolvedSelfAlias)
                        }
                        PredicateColumn::OtherAliased(_) => {
                            return Err(ResolverError::UnresolvedOtherAlias)
                        }
                        PredicateColumn::Literal(l) => stmt.push_str(l),
                        PredicateColumn::Aliased(canonical_alias, col) => {
                            let alias = alias_translator.translate(canonical_alias);
                            stmt.push_str(&alias);
                            stmt.push('.');
                            stmt.push_str(col);
                        }
                    };

                    match a.len() {
                        0 => return Err(ResolverError::ArgumentMissing),
                        1 => {
                            stmt.push_str(" =  ?");
                            args.push(a.get(0).unwrap().to_owned());
                        }
                        _ => {
                            stmt.push_str(" IN (?");
                            for _ in 1..a.len() {
                                stmt.push_str(", ?");
                            }
                            stmt.push(')');
                            args.extend(a.to_owned());
                        }
                    }
                }
                _ => {
                    for ar in a {
                        for c in columns {
                            match c {
                                PredicateColumn::SelfAliased(_) => {
                                    return Err(ResolverError::UnresolvedSelfAlias)
                                }
                                PredicateColumn::OtherAliased(_) => {
                                    return Err(ResolverError::UnresolvedOtherAlias)
                                }
                                PredicateColumn::Literal(lit) => stmt.push_str(lit),
                                PredicateColumn::Aliased(canonical_alias, col) => {
                                    let alias = alias_translator.translate(canonical_alias);
                                    stmt.push_str(&alias);
                                    stmt.push('.');
                                    stmt.push_str(col);
                                }
                            };

                            stmt.push_str(" = ? AND ");
                            args.push(ar.to_owned());
                        }
                        // Remove ' AND '
                        stmt.pop();
                        stmt.pop();
                        stmt.pop();
                        stmt.pop();
                        stmt.pop();
                        stmt.push_str(" OR ");
                    }
                    // Remove ' OR '
                    stmt.pop();
                    stmt.pop();
                    stmt.pop();
                    stmt.pop();
                }
            },
        }
        Ok(())
    }
}

impl Default for Resolver<'_> {
    fn default() -> Self {
        Self::new()
    }
}
