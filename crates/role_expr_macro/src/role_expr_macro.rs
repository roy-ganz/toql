use syn::parse::{Parse, ParseStream};

use syn::{Error,  Ident,  LitStr, Result, Token};

use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use toql_role_expr_parser::PestRoleExprParser;

use pest::{  Parser};
use pest::iterators::Pair;

use toql_role_expr_parser::Rule;

#[derive(Debug)]
pub struct RoleExprMacro {
    pub query: LitStr,
}

impl Parse for RoleExprMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RoleExprMacro {
            query: input.parse()?,
        })
    }
}


pub fn parse(
    role_expr_string: &LitStr,
) -> std::result::Result<TokenStream, TokenStream> {
    
        

   
        fn evaluate_pair(pair: Pair<Rule>) -> Option<TokenStream> {
            /*  println!("Rule:    {:?}", pair.as_rule());
            println!("Span:    {:?}", pair.as_span());
            println!("Text:    {}", pair.as_str()); */

            let span = pair.clone().as_span();

            match pair.as_rule() {
                Rule::role => { 
                                let role = span.as_str();
                                Some( quote!(toql::role_expr::RoleExpr::role(#role . to_string())))
                                },
                Rule::and_clause => {
                    let mut expr: Option<TokenStream> = None;
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
                                         quote!(toql::role_expr::RoleExpr::Not(Box::new(#r)))
                                    } else {
                                       quote!(#r)
                                    };

                                    expr = Some(quote!(toql::role_expr::RoleExpr::And(Box::new(#ex), Box::new(#e))));
                                },
                                None => expr = 
                                   if negate {
                                       negate = false;
                                    Some(quote!(toql::role_expr::RoleExpr::Not(Box::new(#r))))
                                   } else {
                                    Some(quote!(#r))
                                   }
                            }
                        }
                    }
                    expr
                }
                Rule::or_clause => {
                    let mut negate = false;
                    let mut expr: Option<TokenStream> = None;
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
                                         quote!(toql::role_expr::RoleExpr::Not(Box::new(#r)))
                                    } else {
                                       quote!(#r)
                                    };

                                    expr = Some(quote!(toql::role_expr::RoleExpr::Or(Box::new(#ex), Box::new(#e))));
                                    
                                },
                                None => {
                                    expr = if negate {
                                       negate = false;
                                        Some(quote!(toql::role_expr::RoleExpr::Not(Box::new(#r))))
                                   } else {
                                        Some(quote!(#r))
                                   }
                                }
                            }
                        }
                    }
                    expr
                }
                _ => None,
            }
        }


         

    // eprintln!("About to parse {}", toql_string);
    match PestRoleExprParser::parse(Rule::or_clause, &role_expr_string.value()) {
        Ok(pairs) => {
             let mut expr: Option<TokenStream> = None;
            for p in pairs {
                let e = evaluate_pair(p);
                match (&expr, &e) {
                    (Some(ex), Some(e)) => {
                        expr = Some(quote!(toql::role_expr::RoleExpr::Or(Box::new(#ex)), Box::new(#e)));
                    }
                    (None, Some(e)) => {
                        expr = Some(quote!(#e));
                    }
                    _ => {}
                }
            }
            
            Ok(match expr
            {
                Some(e) => e,
                None => quote!()
            })
        }
        Err(e) => {
            let msg = e.to_string();
            Err(quote_spanned!(role_expr_string.span() => compile_error!(#msg)))
        }
    }

  

      
}

    

