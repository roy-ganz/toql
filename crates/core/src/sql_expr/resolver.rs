
use crate::{sql_arg::SqlArg, parameter::ParameterMap, sql_expr::{SqlExprToken, SqlExpr}, sql::Sql};
use super::resolver_error::ResolverError;
use std::borrow::Cow;

pub struct Resolver<'a> {
    self_alias: Option<&'a str>,
    other_alias: Option<&'a str>,
    arguments:  Option<&'a [SqlArg]>,
    aux_params: Option<&'a ParameterMap<'a>>
}

impl<'a> Resolver<'a> {

    pub fn new() -> Self {
        Self {
            self_alias: None,
            other_alias:None,
            arguments: None,
            aux_params: None
        }
    }

    pub fn with_self_alias(&mut self, alias: &'a str) -> &mut Self{
        self.self_alias= Some(alias);
        self
    }
    pub fn with_other_alias(&mut self, alias: &'a str) -> &mut Self{
        self.other_alias= Some(alias);
        self
    }
    pub fn with_aux_params(&mut self, aux_params: &'a ParameterMap<'a>) -> &mut Self{
        self.aux_params= Some(aux_params);
        self
    }
    pub fn with_arguments(&mut self, arguments: &'a [SqlArg]) -> &mut Self{
        self.arguments= Some(arguments);
        self
    }

   

    pub fn resolve(&self, sql_expr: &'a SqlExpr) -> std::result::Result<SqlExpr, ResolverError> {
        
        let mut tokens = Vec::new();
        let mut arg_iter =  if self.arguments.is_some() {
            Some(self.arguments.unwrap())
        }else {
            None
        }; 

        for token in sql_expr.tokens() {
            tokens.push(self.resolve_token(token)?.into_owned())
        }  
        

        Ok(SqlExpr::from(tokens))

    }


    pub fn into_sql(&self, sql_expr: SqlExpr) -> Result<Sql, ResolverError>{

        let stmt= String::new();
        let args :Vec<SqlArg>= Vec::new();

        for unresolved_token in &sql_expr.tokens {
            let token = self.resolve_token(unresolved_token)?;
            Self::token_to_sql(&token, &mut stmt, &mut args);
        }

        Ok(Sql(stmt, args))
   }

    fn resolve_token(&self, token: &'a SqlExprToken) -> Result<Cow<'a, SqlExprToken>, ResolverError> {

         
        let mut arg_iter =  if self.arguments.is_some() {
            Some(self.arguments.unwrap())
        }else {
            None
        }; 

        match token {
               
                SqlExprToken::SelfAlias if self.self_alias.is_some() => {
                   Ok(Cow::Owned(SqlExprToken::Alias(self.self_alias.unwrap().to_string())))
                }
                SqlExprToken::OtherAlias if self.other_alias.is_some() => {
                   Ok(Cow::Owned(SqlExprToken::Alias(self.other_alias.unwrap().to_string())))
                }
                SqlExprToken::AuxParam(name) if self.aux_params.is_some() => {
                    let arg = self.aux_params.unwrap()
                            .get(&name)
                            .ok_or(ResolverError::AuxParamMissing(name.to_string()))?
                            .to_owned();
                   Ok(Cow::Owned(SqlExprToken::Arg(arg)))
                }
                SqlExprToken::UnresolvedArg if arg_iter.is_some()  => {
                     let arg = arg_iter
                     .unwrap()
                     .iter()
                     .next()
                     .ok_or(ResolverError::ArgumentMissing)?;
                                                
                   Ok(Cow::Owned(SqlExprToken::Arg(arg.to_owned())))
                }
                tok @ _ => {
                   Ok(Cow::Borrowed(tok))
                }
            }  


    }

    pub fn token_to_sql(token: &SqlExprToken, mut stmt: &mut String, mut args: &mut Vec<SqlArg>) -> std::result::Result<(), ResolverError> {
        let mut aliased = false;
        
       
            match token {
                   SqlExprToken::SelfAlias => {
                    return Err(ResolverError::UnresolvedSelfAlias)
                },
                   SqlExprToken::OtherAlias => {
                    return Err(ResolverError::UnresolvedOtherAlias)
                },
                   SqlExprToken::UnresolvedArg => {
                    return Err(ResolverError::UnresolvedArgument)
                },
                   SqlExprToken::AuxParam(name) => {
                    return Err(ResolverError::UnresolvedAuxParameter(name.to_owned()))
                },

                SqlExprToken::Literal(lit) => {
                    if aliased && !lit.starts_with(' ') {
                        stmt.push('.');
                        aliased = false;
                    }
                    stmt.push_str(&lit)
                }
             
                 SqlExprToken::Alias(alias) => {
                    stmt.push_str(alias);
                    aliased = true
                },
             
                SqlExprToken::Arg(arg) => {
                    stmt.push_str("?");
                    args.push(arg.to_owned());
                    
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
                        args.push(args.get(0).unwrap().to_owned());
                    }
                    _ => {
                        if aliased && !column.starts_with(' ') {
                            stmt.push('.');
                            aliased = false;
                        }
                        stmt.push_str(" IN (");
                        for a in args {
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
