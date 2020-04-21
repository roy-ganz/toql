//!
//! The query parser can turn a string that follows the Toql query syntax into a [Query](../query/struct.Query.html).
//!
//! ## Example
//!
//! ``` ignore
//! let  query = QueryParser::parse("*, +username").unwrap();
//! assert_eq!("*, +username", query.to_string());
//! ```
//! Read the guide for more information on the query syntax.
//!
//! The parser is written with [Pest](https://pest.rs/) and is fast. It should be used to parse query request from users.
//! To build a query within your program, build it programmatically with the provided methods.
//! This avoids typing mistakes and - unlike parsing - cannot fail.
//!
use crate::error::ToqlError;
use crate::sql_builder::SqlBuilderError;
use crate::query::Concatenation;
use crate::query::Field;
use crate::query::Predicate;
use crate::query::FieldFilter;
use crate::query::FieldOrder;
use crate::query::Query;
use crate::query::QueryToken;
use crate::query::Wildcard;
use crate::sql::SqlArg;
use pest::error::Error;
use pest::error::ErrorVariant::CustomError;
use pest::Parser;


#[derive(Parser)]
#[grammar = "toql.pest"]
struct PestQueryParser;

/// The query parser.
/// It contains only a single static method to turn a string into a Query struct.
pub struct QueryParser;

enum TokenType {
    Field,
    Wildcard,
    Predicate,
    Unknown
}

struct TokenInfo {
    token_type: TokenType,
    args :Vec<SqlArg>,
    hidden : bool,
    order :Option<FieldOrder>,
    filter: Option<String>,
    aggregation :bool,
    name: String,
    concatenation: Concatenation
}

impl TokenInfo {
    pub fn new() -> Self {
        TokenInfo {
            token_type: TokenType::Unknown,
            args : Vec::new(),
            hidden : false,
            order : None,
            filter:  None,
            aggregation : false,
            name: String::new(),
            concatenation: Concatenation::And
       }
       }
       pub fn build_token(&mut self) ->  Result<Option<QueryToken>, ToqlError> {
           match &self.token_type {
               TokenType::Field => {
                   Ok(Some(QueryToken::Field (Field{
                     name: self.name.to_string(),
                    hidden: self.hidden,
                    order: self.order.clone(),
                    filter: self.build_filter()?,
                    aggregation :  self.aggregation,
                    concatenation : self.concatenation.clone()
                   })))

               },
                TokenType::Wildcard => {
                   Ok(Some(QueryToken::Wildcard (Wildcard{
                     path: self.name.to_string(),
                    concatenation : self.concatenation.clone()
                   })))

               },
                 TokenType::Predicate => {
                   Ok(Some(QueryToken::Predicate (Predicate{
                     name: self.name.to_string(),
                     args: self.args.drain(..).collect(),
                    concatenation : self.concatenation.clone()
                   })))

               },
               _ => Ok(None)
           }
       }

       pub fn build_filter(&mut self) -> Result<Option<FieldFilter>, ToqlError>{
           match &self.filter {
               Some(f) => {
                   let upc = f.to_uppercase();
                   let filtername = upc.split_ascii_whitespace().next().unwrap_or("");
                   match filtername {
                    "" => Ok(None),
                    "EQ" => Ok(Some(FieldFilter::Eq(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "EQN" => Ok(Some(FieldFilter::Eqn)),
                    "NE" => Ok(Some(FieldFilter::Ne(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "NEN" => Ok(Some(FieldFilter::Nen)),
                    "GT" => Ok(Some(FieldFilter::Gt(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "GE" => Ok(Some(FieldFilter::Ge(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "LT" => Ok(Some(FieldFilter::Lt(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "LE" => Ok(Some(FieldFilter::Le(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "LK" => Ok(Some(FieldFilter::Lk(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "IN" => Ok(Some(FieldFilter::In(self.args.drain(..).collect()))),
                    "OUT" => Ok(Some(FieldFilter::Out(self.args.drain(..).collect()))),
                    "BW" => Ok(Some(FieldFilter::Bw(  self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?, self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                    "RE" => Ok(Some(FieldFilter::Re(self.args.pop().ok_or(SqlBuilderError::FilterInvalid(filtername.to_string()))?))),
                   
                    _ => if f.starts_with("FN ") {
                            let filtername = f.trim_start_matches("FN ").to_uppercase();
                            Ok(Some(FieldFilter::Fn(filtername.to_string(),self.args.drain(..).collect())))

                        } else {
                            Err(SqlBuilderError::FilterInvalid(f.to_string()).into())
                        } 
                   }
               },
               None => Ok(None)
          }
       }
}

impl QueryParser {
    /// Method to parse a string
    /// This fails if the syntax is wrong. The original PEST error is wrapped with the ToqlError and
    /// can be used to examine to problem in detail.
    pub fn parse<M>(toql_string: &str) -> Result<Query<M>, ToqlError> {
        let pairs = PestQueryParser::parse(Rule::query, toql_string)?;

        let mut query = Query::new();
        
        let mut token_info = TokenInfo::new();
        
        for pair in pairs.flatten().into_iter() {
            let span = pair.clone().as_span();
            //   println!("Rule:    {:?}", pair.as_rule());
            //   println!("Span:    {:?}", span);
            //   println!("Text:    {}", span.as_str());
            match pair.as_rule() {
                Rule::field_clause => {
                     token_info.token_type = TokenType::Field;
                },
                Rule::sort => {
                        let p = span.as_str()[1..].parse::<u8>().unwrap_or(1);
                        if let Some('+') = span.as_str().chars().next() {
                                token_info.order = Some(FieldOrder::Asc(p));
                            } else {
                                 token_info.order = Some(FieldOrder::Desc(p));
                            }
                }
                Rule::hidden => {
                    token_info.hidden = true;
                }
                Rule::aggregation => {
                    token_info.aggregation = true;
                }
                
                Rule::field_path => {
                     
                     token_info.name = span.as_str().to_string();
                },
                 Rule::wildcard => {
                    token_info.token_type = TokenType::Wildcard;
                }
                Rule::wildcard_path => {
                      token_info.name = span.as_str().to_string();
                }
                Rule::filter_name => {
                    token_info.filter =  Some(span.as_str().to_string());
                },
                Rule::num_u64 => {
                    let v = span.as_str().parse::<u64>().unwrap_or(0); // should not be invalid, todo check range
                    token_info.args.push(SqlArg::from(v));
                }
                Rule::num_i64 => {
                    let v = span.as_str().parse::<u64>().unwrap_or(0); // should not be invalid, todo check range
                    token_info.args.push(SqlArg::from(v));
                }
                Rule::num_f64 => {
                    let v = span.as_str().parse::<u64>().unwrap_or(0); // should not be invalid, todo check range
                    token_info.args.push(SqlArg::from(v));
                }
                Rule::string => {
                    let v = span.as_str().trim_start_matches("'").trim_end_matches("'").replace("''", "'");
                    token_info.args.push(SqlArg::from(v));
                }
                 Rule::predicate_clause => {
                     token_info.token_type= TokenType::Predicate;
                },
                 Rule::predicate_name =>  {
                     token_info.name = span.as_str().trim_start_matches("@").to_string();
                },
                Rule::rpar => {
                    query.tokens.push(QueryToken::RightBracket);
                }
                Rule::lpar => {
                    query.tokens.push(QueryToken::LeftBracket(token_info.concatenation.clone()));
                }
                Rule::concat => {

                let concat_type =  span.as_str().chars().next();
                   if let Some(token) = token_info.build_token()?
                  /*  .map_err(|e| ToqlError::QueryParserError(Error::new_from_span (
                                        CustomError {    message:e      }, span)))?  */
                                        {
                        query.tokens.push(token);
                        token_info = TokenInfo::new(); // Restart token builder

                       // println!("{:?}", query);
                    }

                    token_info.concatenation = if let Some(',') = concat_type {
                         Concatenation::And
                    } else {
                        Concatenation::Or
                    };

                }

                _ => {}
            }
        }
        if let Some(token) = token_info.build_token()? // TODO error handling
        {
            query.tokens.push(token);
        }
      //  println!("{:?}", query);
        Ok(query)
    }
}
