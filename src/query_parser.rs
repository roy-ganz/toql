use crate::query::Concatenation;
use crate::query::Field;
use crate::query::FieldFilter;
use crate::query::FieldOrder;
use crate::query::Query;
use crate::query::QueryToken;
use pest::Parser;
use pest::error::Error;


#[derive(Parser)]
#[grammar = "toql.pest"]
struct PestQueryParser;

pub struct QueryParser;

impl QueryParser {

   
    pub fn parse(toql_string: &str) -> Result<Query, Error<Rule>>  {
        let pairs = PestQueryParser::parse(Rule::query, toql_string)?;

        let mut query = Query::new();
        let mut con = Concatenation::And;
       
        for pair in pairs.flatten().into_iter() {
            let span = pair.clone().into_span();
         //   println!("Rule:    {:?}", pair.as_rule());
         //   println!("Span:    {:?}", span);
         //   println!("Text:    {}", span.as_str());
            match pair.as_rule() {
                // Rule::wclause => {
                //     query.tokens.push(QueryToken::Field(Field {
                //         concatenation: con.clone(),
                //         name: "missing".to_string(),
                //         hidden: false,
                //         order: None,
                //         having: false,
                //         filter: None,
                //         dynamic: false,
                //     }));
                // }
                Rule::field_clause => {
                    query.tokens.push(QueryToken::Field(Field {
                        concatenation: con.clone(),
                        name: "missing".to_string(),
                        hidden: false,
                        order: None,
                        aggregation: false,
                        filter: None,
                    }));
                }
                Rule::sort => {
                    let token = query.tokens.last_mut();
                    if let Some(t) = token {
                        if let QueryToken::Field(ref mut field) = t {
                            let p = span.as_str()[1..].parse::<u8>().unwrap_or(1);
                            if let Some('+') = span.as_str().chars().next() {
                                field.order = Some(FieldOrder::Asc(p));
                            } else {
                                field.order = Some(FieldOrder::Desc(p));
                            }
                        }
                    }
                }
                Rule::hidden => {
                    let token = query.tokens.last_mut();
                    if let Some(t) = token {
                        if let QueryToken::Field(ref mut field) = t {
                            field.hidden = true;
                        }
                    }
                }
                Rule::aggregation => {
                    let token = query.tokens.last_mut();
                    if let Some(t) = token {
                        if let QueryToken::Field(ref mut field) = t {
                            field.aggregation = true;
                        }
                    }
                }
               
                Rule::fieldname => {
                    let token = query.tokens.last_mut();
                    if let Some(t) = token {
                        if let QueryToken::Field(ref mut field) = t {
                            field.name = span.as_str().to_string();
                        }
                    }
                }
                Rule::field_filter => {
                    let token = query.tokens.last_mut();
                    if let Some(t) = token {
                        if let QueryToken::Field(ref mut field) = t {
                            let mut iter = span.as_str().split_whitespace();
                            let op = iter.next();

                            field.filter = match op.unwrap().to_uppercase().as_str() {
                                "EQ" => Some(FieldFilter::Eq(iter.next().unwrap().to_string())),
                                "NE" => Some(FieldFilter::Ne(iter.next().unwrap().to_string())),
                                "GT" => Some(FieldFilter::Gt(iter.next().unwrap().to_string())),
                                "GE" => Some(FieldFilter::Ge(iter.next().unwrap().to_string())),
                                "LT" => Some(FieldFilter::Lt(iter.next().unwrap().to_string())),
                                "LE" => Some(FieldFilter::Le(iter.next().unwrap().to_string())),
                                "LK" => Some(FieldFilter::Lk(iter.next().unwrap().to_string())),
                                "IN" => Some(FieldFilter::In(iter.map(String::from).collect())),
                                "OUT" => Some(FieldFilter::Out(iter.map(String::from).collect())),
                                _ => Some(FieldFilter::Other(iter.map(String::from).collect())),
                            }
                        }
                    }
                }
                Rule::wildcard => {
                    query.tokens.push(QueryToken::Wildcard);
                }
                 Rule::rpar => {
                                         
                    query.tokens.push(QueryToken::RightBracket);
                }
                Rule::lpar => {
                    query.tokens.push(QueryToken::LeftBracket(con.clone()));
                }
                Rule::concat => {
                    if let Some(',') = span.as_str().chars().next() {
                        con = Concatenation::And;
                    } else {
                        con = Concatenation::Or;
                    }
                }

                _ => {}
            }
            //println!("{:?}", query.tokens);
        }
        Ok(query)
    }
}
