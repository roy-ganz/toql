use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{ Expr, LitStr, Result, Token};

use proc_macro2::{ TokenStream};
use toql_sql_expr_parser::PestSqlExprParser;

use pest::Parser;

use toql_sql_expr_parser::Rule;


#[derive(Debug)]
pub struct SqlExprMacro {
    pub query: LitStr,
    pub arguments: Punctuated<Expr, Token![,]>,
}

impl Parse for SqlExprMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(SqlExprMacro {
            query: input.parse()?,
            arguments: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![,]) {
                    // arguments ?
                    input.parse::<Token![,]>()?; // skip ,
                    input.parse_terminated(Expr::parse)?
                } else {
                    Punctuated::new()
                }
            },
        })
    }
}



#[derive(Debug)]
struct FieldInfo {
    pub literal: String,
}

impl FieldInfo {
    pub fn new() -> Self {
        FieldInfo {
            literal: String::new(),
        }
    }
}


pub fn parse(
    sql_expr_string: &LitStr,
    expr_args: &mut syn::punctuated::Iter<'_, syn::Expr>,
) -> std::result::Result<TokenStream, TokenStream> {
   

    // eprintln!("About to parse {}", toql_string);
    match PestSqlExprParser::parse(Rule::query, &sql_expr_string.value()) {
        Ok(pairs) => {
            Ok(evaluate_pair(
                &mut pairs.flatten().into_iter(),
                expr_args,
            )?)
        }
        Err(e) => {
            let msg = e.to_string();
            Err(quote_spanned!(sql_expr_string.span() => compile_error!(#msg)))
        }
    }
  
}

fn append_literal(field_info : &mut FieldInfo, tokens: &mut Vec<TokenStream>) {
    let lit = &field_info.literal;
    if !lit.is_empty() {
        tokens.push( quote!(  toql::sql_expr::SqlExprToken::Literal(String::from(#lit))));
        field_info.literal.clear();
    }
}
fn append_tokens( tokens: &mut Vec<TokenStream>, outstream: &mut TokenStream) {
    
   
    if !tokens.is_empty(){
         if outstream.is_empty() {
            outstream.extend(quote!( t ));
        } 
        outstream.extend(quote!(.extend(toql::sql_expr::SqlExpr::from(vec![ #(#tokens),* ]))));
    }
}


fn evaluate_pair(
    pairs: &mut pest::iterators::FlatPairs<toql_sql_expr_parser::Rule>,
    expr_args: &mut syn::punctuated::Iter<'_, syn::Expr>,
) -> std::result::Result<TokenStream, TokenStream> {
    
    

    let mut with_args = false;
    let mut alias = false;
    let mut field_info = FieldInfo::new();
    let mut outstream : TokenStream = TokenStream::new(); // actual output stream

    let mut tokens : Vec<TokenStream> = Vec::new(); // collect tokens for vec
    
    
    
    
    while let Some(pair) = pairs.next() {
        let span = pair.clone().as_span();
    

        match pair.as_rule() {
            
             Rule::placeholder => {
                append_literal(&mut field_info, &mut tokens);
                append_tokens(&mut tokens, &mut outstream);
                tokens.clear();
                 if let Some(a) = expr_args.next() {
                    if outstream.is_empty() {
                        outstream.extend(quote!( t ));
                    }
                    outstream.extend( quote!( .extend(#a)));
                 } else {
                    return Err(quote!(compile_error!("Missing placeholder argument")));
                 }
                alias = false;
            } 
            Rule::self_alias => {
                append_literal(&mut field_info, &mut tokens);
                tokens.push( quote!( toql::sql_expr::SqlExprToken::SelfAlias));
                alias = true;
            }
            Rule::other_alias => {
               append_literal(&mut field_info, &mut tokens);
               tokens.push( quote!( toql::sql_expr::SqlExprToken::OtherAlias));
               alias = true;

            }
            Rule::quoted => {
               let text = span.as_str();
               field_info.literal.push_str(text);
               alias = false;

            }
            Rule::aux_param => {
               append_literal(&mut field_info, &mut tokens);
               let name = span.as_str().trim_start_matches("<").trim_end_matches(">");
               alias = false;
               tokens.push( quote!( toql::sql_expr::SqlExprToken::AuxParam(String::from(#name))));
                
            }
            Rule::literal => {

                // Add a dot if an alias immediately precedes a non whitespace literal (..column_name)
                let l = span.as_str();

                if alias == true && l != " " {
                    field_info.literal.push('.');
                }
                // If literal is ? insert arguments
                if l == "?" {
                    append_literal(&mut field_info, &mut tokens);
                    if let Some(a) = expr_args.next() {
                        tokens.push( quote!( toql::sql_expr::SqlExprToken::Arg(toql::sql_arg::SqlArg::from(#a))));
                        with_args= true;
                    } else {
                        if with_args == false {
                           tokens.push( quote!( toql::sql_expr::SqlExprToken::UnresolvedArg));
                        } else {
                            return Err(quote!(compile_error!("Missing value for argument")));
                        }
                    }
                } else {
                    field_info.literal.push_str(l)
                }
                alias = false;
            }

            _ => {}
        }
    }
    

    append_literal(&mut field_info, &mut tokens);
    if  expr_args.next().is_some() {
        return Err(quote!(compile_error!("To many values for arguments");));
    }
    append_tokens(&mut tokens, &mut outstream);

    

    Ok(quote!({let mut t = toql::sql_expr::SqlExpr::new(); #outstream; t})) // return value not reference
}
