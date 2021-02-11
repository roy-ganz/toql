use syn::parse::{Parse, ParseStream};

use syn::{ Type, Ident, LitStr, Result, Token};

use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use toql_field_list_parser::PestFieldListParser;

use pest::Parser;

use toql_field_list_parser::Rule;


#[derive(Debug)]
pub enum FieldsMacro {
   FieldList { struct_type: Type, query: LitStr },
   Top,
}


fn parse_fieldlist(input: ParseStream) -> Result<FieldsMacro> {
        Ok(FieldsMacro::FieldList {
                struct_type: { (input.parse()?, input.parse::<Token![,]>()?).0 },
                query: input.parse()?,
            })
}

fn parse_top(input: ParseStream) -> Result<FieldsMacro> {
        let top: Ident = input.parse()?;
            if top.to_string() == "top" {
                Ok(FieldsMacro::Top)
            } else {
                println!("Found {}", top);
                Err(syn::Error::new(
                    Span::call_site(),
                    "Keyword `top` expected",
                ))
            }
}


impl Parse for FieldsMacro {
    fn parse(input: ParseStream) -> Result<Self> {

        let alt = input.fork();
        let f = parse_fieldlist(input);
        match f {
            Ok(_) => {f}
            Err(_) => {parse_top(&alt)}
        }
    }
}


pub fn parse(
    field_list_string: &LitStr,
    struct_type: Type,
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
    struct_type: &Type,
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
               // methods.push( quote!(toql::update_field::UpdateField::into_field( toql::query_path::QueryPath::wildcard(<#struct_type as toql::query_fields::QueryFields>::fields() #(#method_names)*) )  ))
                methods.push( quote!(toql::query_path::QueryPath::wildcard(<#struct_type as toql::query_fields::QueryFields>::fields() #(#method_names)*).into_string()) )
                
            }
            Rule::field_path => {
                let method_names = span.as_str().split("_")
                    .filter(|p| !p.is_empty())
                    .map(|p| {
                         let raw_string= format!("r#{}", &p.to_snake_case());
                        let mut raw_ident  :Ident = syn::parse_str(&raw_string).unwrap();
                        raw_ident.set_span(Span::call_site());
                        //let name = Ident::new(&format!("r#{}", &p.to_snake_case()),Span::call_site());
                      //  let p = proc_macro2::Punct::new('#', proc_macro2::Spacing::Joint);
                     
                        quote!( .  #raw_ident ())
                    })
                    .collect::<Vec<_>>();
                //methods.push(  quote!(toql::update_field::UpdateField::into_field(  <#struct_type as toql::query_fields::QueryFields>::fields() #(#method_names)*  )))
                methods.push(  quote!( <#struct_type as toql::query_fields::QueryFields>::fields() #(#method_names)*  .into_name()))
                
            }
            _ => {}
        }
    }

   
   Ok(quote!(
      //  toql::fields::Fields::<#struct_type>::from(vec![#(#methods),* ])
      toql::backend::fields::Fields::from(vec![#(#methods),* ])
    ))
    

    
}
