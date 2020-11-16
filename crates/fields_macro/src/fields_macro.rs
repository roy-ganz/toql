use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, Ident, Lit, LitStr, Result, Token};

use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use toql_field_list_parser::PestFieldListParser;

use pest::Parser;

use toql_field_list_parser::Rule;

#[derive(Debug)]
enum TokenType {
    Field,
    Wildcard,
    Predicate,
    Query,
    Unknown,
}

#[derive(Debug)]
pub struct FieldsMacro {
    pub ident: Ident,
    pub query: LitStr,
}

impl Parse for FieldsMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldsMacro {
            ident: { (input.parse()?, input.parse::<Token![,]>()?).0 },
            query: input.parse()?,
        })
    }
}


pub fn parse(
    field_list_string: &LitStr,
    struct_type: Ident,
) -> std::result::Result<TokenStream, TokenStream> {
    let mut output_stream: TokenStream = TokenStream::new();

    match PestFieldListParser::parse(Rule::query, &field_list_string.value()) {
        Ok(pairs) => {
            output_stream.extend(evaluate_pair(
                &mut pairs.flatten().into_iter(),
                &struct_type
            )?);
        }
        Err(e) => {
            let msg = e.to_string();
            return Err(quote_spanned!(field_list_string.span() => compile_error!(#msg)));
        }
    };

    Ok(output_stream)
}

fn evaluate_pair(
    pairs: &mut pest::iterators::FlatPairs<toql_field_list_parser::Rule>,
    struct_type: &Ident,
) -> std::result::Result<TokenStream, TokenStream> {
    
    

   
     let mut methods = Vec::new();

    while let Some(pair) = pairs.next() {
        let span = pair.clone().as_span();

   
   
   
        match pair.as_rule() {
           
            Rule::wildcard => {
                let method_names = span.as_str().split("_")
                    .filter(|p| !p.is_empty())
                     .filter(|p| *p != "*")
                    .map(|p| {
                        let name = Ident::new(&format!("{}", &p.to_snake_case()), Span::call_site());
                        quote!( .#name ())
                    })
                    .collect::<Vec<_>>();
                methods.push( quote!(toql::update_field::UpdateField::into_field(  #struct_type::fields() #(#method_names)* .wildcard() )  ))
                
            }
            Rule::field_path => {
                let method_names = span.as_str().split("_")
                    .filter(|p| !p.is_empty())
                    .map(|p| {
                        let name = Ident::new(&format!("{}", &p.to_snake_case()), Span::call_site());
                        quote!( . #name ())
                    })
                    .collect::<Vec<_>>();
                methods.push(  quote!(toql::update_field::UpdateField::into_field(  #struct_type::fields() #(#method_names)*  )))
                
            }
            _ => {}
        }
    }

   
   Ok(quote!(
        Fields::<#struct_type>::from(vec![#(#methods),* ])
    ))
    

    
}
