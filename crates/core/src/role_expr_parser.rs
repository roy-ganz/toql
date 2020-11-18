use crate::pest::Parser;
use toql_role_expr_parser::{PestRoleExprParser, Rule};

use crate::error::ToqlError;
use crate::{role_expr::RoleExpr};

pub struct RoleExprParser;

impl RoleExprParser {
    /// Method to parse a string
    /// This fails if the syntax is wrong. The original PEST error is wrapped with the ToqlError and
    /// can be used to examine to problem in detail.
    pub fn parse(role_expr: &str) -> Result<RoleExpr, ToqlError> {
        let pairs = PestRoleExprParser::parse(Rule::query, role_expr)?;

        let mut exprs : Vec<RoleExpr> = Vec::new();
        
        for pair in pairs.flatten().into_iter() {
            let span = pair.clone().as_span();
            //   println!("Rule:    {:?}", pair.as_rule());
            //   println!("Span:    {:?}", span);
            //   println!("Text:    {}", span.as_str());
            match pair.as_rule() {
                 Rule::role => {
                     exprs.push(RoleExpr::role(span.as_str().to_string()));
                 }
                  Rule::not => {
                      if let Some(e) = exprs.pop() {
                        exprs.push(RoleExpr::Not(Box::new(e)));
                      }
                  }
                  Rule::separator => {
                      let concat = span.as_str().chars().next().unwrap_or(',');
                      if exprs.len() >= 2 {
                            let a = exprs.pop().unwrap();
                            let b = exprs.pop().unwrap();
                            if concat == ',' {
                                exprs.push(RoleExpr::And(Box::new(a), Box::new(b)));
                            } else {
                                exprs.push(RoleExpr::Or(Box::new(a), Box::new(b)));
                            };
                    }
                  }

                /*
                Rule::literal => {
                    // If last token is literal append to that
                    if let Some(SqlExprToken::Literal(l)) = tokens.last_mut() {
                        l.push_str(span.as_str());
                    } else {
                        tokens.push(SqlExprToken::Literal(span.as_str().to_string()))
                    }
                }
                Rule::quoted => tokens.push(SqlExprToken::Literal(span.as_str().to_string())),
                Rule::self_alias => {
                    tokens.push(SqlExprToken::SelfAlias);
                    tokens.push(SqlExprToken::Literal(".".to_string()))
                }
                Rule::other_alias => {
                    tokens.push(SqlExprToken::OtherAlias);
                    tokens.push(SqlExprToken::Literal(".".to_string()))
                }
                Rule::aux_param => tokens.push(SqlExprToken::AuxParam(span.as_str().to_string())),
                */

                _ => {}
            }
        }

        //  println!("{:?}", query);
        Ok(exprs.pop().unwrap_or(RoleExpr::invalid()))
    }
}
