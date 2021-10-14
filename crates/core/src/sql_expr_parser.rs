use crate::pest::Parser;
use toql_sql_expr_parser::{PestSqlExprParser, Rule};

use crate::error::ToqlError;
use crate::sql_expr::{SqlExpr, SqlExprToken};

pub struct SqlExprParser;

impl SqlExprParser {
    /// Method to parse a string
    /// This fails if the syntax is wrong. The original PEST error is wrapped with the ToqlError and
    /// can be used to examine to problem in detail.
    pub fn parse(sql_expr: &str) -> Result<SqlExpr, ToqlError> {
        let pairs = PestSqlExprParser::parse(Rule::query, sql_expr)?;

        let mut tokens: Vec<SqlExprToken> = Vec::new();

        for pair in pairs.flatten().into_iter() {
            let span = pair.clone().as_span();
            match pair.as_rule() {
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

                _ => {}
            }
        }

        Ok(SqlExpr::from(tokens))
    }
}
