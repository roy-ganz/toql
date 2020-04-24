

use syn::{Ident, Lit, LitStr, Result, Token, Expr, Error};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

 use proc_macro2::{TokenStream, Span};
use toql_query_parser::PestQueryParser;
use heck::SnakeCase;

use pest::Parser;

use toql_query_parser::Rule;

#[derive(Debug)]
enum TokenType {
    Field,
    Wildcard,
    Predicate,
    Query,
    Unknown
}


#[derive(Debug)]
pub struct QueryBuilder {
    pub ident: Ident,
    pub query: LitStr,
    pub arguments: Punctuated<Expr, Token![,]>
}


impl Parse for QueryBuilder {
    fn parse(input: ParseStream) -> Result<Self> {

        Ok(QueryBuilder {
            ident: {(input.parse()?,input.parse::<Token![,]>()?).0},
            query: input.parse()?,
            arguments: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![,]) { // arguments ? 
                   input.parse::<Token![,]>()?; // skip , 
                   input.parse_terminated(Expr::parse)?
                } else {
                        Punctuated::new()
                }

            }
        })
    }
}

#[derive(Debug, PartialEq)]
enum Concatenation {
And, Or
}

#[derive(Debug)]
struct FieldInfo {
 pub sort  :TokenStream,
 pub hidden  :TokenStream,
 pub field  :TokenStream,
  pub args  :Vec<TokenStream>,
pub single_array_argument: bool,
 pub name : String,
 pub filter_name : Option<String>,
 pub token_type: TokenType,
 pub concat : Concatenation
}

impl FieldInfo {
    pub fn new() -> Self{
        FieldInfo {
            sort  : quote!(),
            hidden  : quote!(),
            field  : quote!(),
            single_array_argument: false,
            args : Vec::new(),

            name : String::new(),
            filter_name : None,
                
            token_type : TokenType::Unknown,
            concat: Concatenation::And
        }
    }

    pub fn concatenated_token(&self, struct_type: &Ident) -> TokenStream {
      
       
            let token = match  self.token_type {
                            TokenType::Field => {
                                // Separate with underscore, convert to snake_case for rust
                                let fnname = self.name.split("_").map(|n|  syn::parse_str::<Ident>(&format!("r#{}",n.to_snake_case())).unwrap());
                                // let reserved = Ident::new("r#", Span::call_site());
                                  let sort = &self.sort;
                                  let hidden = &self.hidden;
                                  let filter = self.filter();
                                Some(quote!(<#struct_type as toql::query_fields::QueryFields>::fields(). #(#fnname()).*  #sort #hidden #filter))
                            },
                            TokenType::Wildcard => {
                                   let name = &self.name;
                                Some(quote!(#struct_type::wildcard(#name)))
                            }, 
                             TokenType::Query => {
                                   let query = &self.args.get(0);
                                    Some(quote!(toql::query::Query::<#struct_type>::from(#query)))
                            }, 
                            TokenType::Predicate =>{
                                  
                                let args = &self.args;
                                 //let fnname = self.name.split("_").map(|n|  Ident::new(&n.to_snake_case(), Span::call_site()));
                                   let fnname = self.name.split("_").map(|n|  syn::parse_str::<Ident>(&format!("r#{}",n.to_snake_case())).unwrap());
                                   let are =  if self.single_array_argument {quote!(.are( #(#args),* ))} else {quote!(.are( &[#(#args),*] )) };
                                    Some(quote!(<#struct_type as toql::query_fields::QueryFields>::fields(). #(#fnname()).* #are ))
                            },
                            TokenType::Unknown =>None
                        };

                        match token {
                            Some(token) => {if self.concat == Concatenation::And {
                                    quote!(.and(#token))
                                } else {
                                    quote!(.or(#token))
                                }},
                            None => quote!()
                        }
                           

    }
    fn filter(&self) -> TokenStream {
        let args = &self.args;
        match &self.filter_name {
            Some(f) => {
                match f.to_uppercase().as_str() {

                    "EQ" => { quote!(.eq(#(#args),*)) },
                    "EQN" =>{ quote!(.eqn()) },
                    "NE" => {quote!(.ne(#(#args),*)) },
                    "NEN" =>  quote!(.nen()),
                    "GT" => { quote!(.gt(#(#args),*)) },
                    "GE" =>{ quote!(.ge(#(#args),*)) },
                    "LT" =>{ quote!(.lt(#(#args),*)) },
                    "LE" => {quote!(.le(#(#args),*)) },
                    "LK" => {  quote!(.lk(#(#args),*)) },
                    "IN" =>  { if self.single_array_argument {quote!(.ins( #(#args),* ))} else {quote!(.ins( &[#(#args),*] )) }},
                    "OUT" => { if self.single_array_argument {quote!(.out( #(#args),* ))} else {quote!(.out( &[#(#args),*] )) }},
                    "BW" => {  quote!(.bw(#(#args),*)) },
                    "RE" => {  quote!(.re(#(#args),*)) },
                    _ => { if f.starts_with("FN ") 
                         { 
                            let name = f.trim_start_matches("FN ").to_uppercase();
                            let args = &self.args;
                            if self.single_array_argument {quote!(.fnc(#name, #(#args),* ))} 
                            else {quote!(.fnc(#name &[#(#args),*] )) }
                        } else {
                            let error= format!("Invalid filter `{}`.", f);
                            quote!(compile_error!(#error))
                        } 
                   }
                   
                }
            },
            None => quote!()
        }
    }
}


pub fn parse(toql_string: &LitStr, struct_type: Ident, query_args: &mut syn::punctuated::Iter<'_, syn::Expr>) -> std::result::Result<TokenStream, TokenStream> {

        let mut output_stream : TokenStream = quote!(toql::query::Query::<#struct_type>::new());

   // eprintln!("About to parse {}", toql_string);
        match  PestQueryParser::parse(Rule::query, &toql_string.value()) {
            Ok(pairs) => {   
                    output_stream.extend(evaluate_pair(&mut pairs.flatten().into_iter(), &struct_type, query_args)?);
               },
            Err(e) => {
                let msg = e.to_string();
                return Err( quote_spanned!(toql_string.span() => compile_error!(#msg)));
            }
        }; 

        Ok(output_stream)
    }

    fn evaluate_pair(pairs: &mut pest::iterators::FlatPairs<toql_query_parser::Rule>, struct_type: &Ident, query_args: &mut syn::punctuated::Iter<'_, syn::Expr>) ->std::result::Result<TokenStream, TokenStream> {
    //fn evaluate_pair(pairs: &pest::iterators::Pair<'_, toql_parser::Rule>) ->TokenStream2 {

            
        let mut field_info =FieldInfo::new();

        let mut output_stream = quote!();

        
           while let Some(pair)= pairs.next() {
                let span = pair.clone().as_span();

                match pair.as_rule() {
                    Rule::lpar => {
                        
                      let content = evaluate_pair(pairs, struct_type, query_args)?;
                       output_stream.extend( if field_info.concat == Concatenation::And {
                          quote!( .and_parentized(toql::query::Query::<#struct_type>::new() #content))
                      } else {
                            quote!( .or_parentized(toql::query::Query::<#struct_type>::new() #content))
                      });
                      
                },
                Rule::rpar => {
                     break;
                },
                 Rule::sort => {
                        let p = span.as_str()[1..].parse::<u8>().unwrap_or(1);
                        if let Some('+') = span.as_str().chars().next() {
                                field_info.sort= quote!(.asc(#p));
                                
                            } else {
                                 field_info.sort= quote!(.desc(#p));
                            }
                },
                Rule::hidden => {
                     field_info.hidden= quote!(.hidden());
                },
              
                 Rule::wildcard_path => {
                      field_info.name = span.as_str().to_string();
                },
                Rule::filter0_name => {
                    field_info.filter_name =  Some(span.as_str().to_string());
                },
                Rule::filter1_name => {
                    field_info.filter_name =  Some(span.as_str().to_string());
                },
                Rule::filter2_name => {
                    field_info.filter_name =  Some(span.as_str().to_string());
                },
                Rule::filterx_name => {
                    field_info.filter_name =  Some(span.as_str().to_string());
                },
                Rule::num_u64 => {
                    let v = span.as_str().parse::<u64>().unwrap_or(0); // should not be invalid, todo check range
                    field_info.args.push( quote!(#v));
                },
                Rule::num_i64 => {
                    let v = span.as_str().parse::<i64>().unwrap_or(0); // should not be invalid, todo check range
                      field_info.args.push( quote!(#v));
                },
                Rule::num_f64 => {
                    let v = span.as_str().parse::<f64>().unwrap_or(0.0); // should not be invalid, todo check range
                      field_info.args.push( quote!(#v));
                },
                Rule::string => {
                    let v = span.as_str().trim_start_matches("'").trim_end_matches("'").replace("''", "'");
                     field_info.args.push( quote!(#v));
                },
                Rule::num_placeholder => {
                    field_info.single_array_argument = true; // first argument contains whole array
                    let v = query_args.next();
                    match v {
                        Some(v) =>  field_info.args.push( quote!(#v)),
                        None =>   {
                             return Err(quote!(compile_error!("Missing argument for placholder.");));
                        }
                    }
                     
                },
                 Rule::predicate_clause => {
                      field_info.token_type= TokenType::Predicate;
                },
                Rule::field_path => {
                     
                      field_info.name = span.as_str().to_string();
                },
                 Rule::field_clause => {
                      field_info.token_type= TokenType::Field;
                },
                 Rule::predicate_name =>  {
                      field_info.name = span.as_str().trim_start_matches("@").to_string();
                },
                Rule::query_placeholder => {
                    field_info.token_type = TokenType::Query;
                    
                    let v = query_args.next();
                    match v {
                        Some(v) =>   field_info.args.push( quote!(#v)),
                        None => return Err(quote!(compile_error!("Missing argument for placeholder")))
                    };
                    
                },
                Rule::separator => {
                     output_stream.extend(field_info.concatenated_token(struct_type));
                     field_info =FieldInfo::new();
                     field_info.concat =  if span.as_str().chars().next().unwrap_or(',') == ',' {Concatenation::And} else {Concatenation::Or};
                },
                _ => {}


                }
              

           }

             output_stream.extend(field_info.concatenated_token(struct_type));
            
            Ok(output_stream)
    }
