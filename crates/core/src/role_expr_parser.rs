use crate::pest::Parser;
use toql_role_expr_parser::{PestRoleExprParser, Rule};

use crate::error::ToqlError;
use crate::role_expr::RoleExpr;
use pest::iterators::Pair;

#[derive(Clone)]
pub enum Concatenation {
    And,
    Or,
}
pub struct StackItem(Concatenation, RoleExpr);

pub struct RoleExprParser;

impl RoleExprParser {
    /// Method to parse a string
    /// This fails if the syntax is wrong. The original PEST error is wrapped with the ToqlError and
    /// can be used to examine to problem in detail.
    pub fn parse(role_expr: &str) -> Result<RoleExpr, ToqlError> {
        fn evaluate_pair(pair: Pair<Rule>) -> Option<RoleExpr> {
            /*  println!("Rule:    {:?}", pair.as_rule());
            println!("Span:    {:?}", pair.as_span());
            println!("Text:    {}", pair.as_str()); */

            let span = pair.clone().as_span();

            match pair.as_rule() {
                Rule::role => Some(RoleExpr::role(span.as_str().to_string())),
               
                Rule::and_clause => {
                    let mut expr: Option<RoleExpr> = None;
                     let mut negate = false;
                    for p in pair.into_inner() {
                          if p.as_rule() == Rule::negate {
                            negate = true;
                            continue;
                        }
                        let res = evaluate_pair(p);
                        if let Some(r) = res {
                            match expr {
                                Some(ex) => {
                                    let e = if negate {
                                        negate = false;
                                        RoleExpr::Not(Box::new(r))
                                    } else {
                                        r
                                    };
                                    expr = Some(RoleExpr::And(Box::new(ex), Box::new(e)));
                                },
                                None => expr = Some(r),
                            }
                        }
                    }
                    expr
                }
                Rule::or_clause => {
                    let mut negate = false;
                    let mut expr: Option<RoleExpr> = None;
                    for p in pair.into_inner() {
                        if p.as_rule() == Rule::negate {
                            negate = true;
                            continue;
                        }
                        let res = evaluate_pair(p);
                        if let Some(r) = res {
                            match expr {
                                Some(ex) => { 
                                    let e = if negate {
                                        negate = false;
                                        RoleExpr::Not(Box::new(r))
                                    } else {
                                        r
                                    };
                                    expr = Some(RoleExpr::Or(Box::new(ex), Box::new(e)));

                                },
                                None => expr = Some(r),
                            }
                        }
                    }
                    expr
                }
                _ => None,
            }
        }

        let pairs = PestRoleExprParser::parse(Rule::or_clause, role_expr)?;
        let mut expr: Option<RoleExpr> = None;
        for p in pairs {
            let e = evaluate_pair(p);
            match (&expr, &e) {
                (Some(ex), Some(e)) => {
                    expr = Some(RoleExpr::Or(Box::new(ex.clone()), Box::new(e.clone())));
                }
                (None, Some(e)) => {
                    expr = Some(e.clone());
                }
                _ => {}
            }
        }
        //  println!("{:?}", query);
        Ok(expr.unwrap_or(RoleExpr::invalid()))
    }
}
