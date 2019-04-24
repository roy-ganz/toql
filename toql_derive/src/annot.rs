
use crate::codegen::GeneratedToql;

#[cfg(feature = "mysqldb")]
use crate::codegen_mysql::GeneratedMysql;

use syn::Ident;
use syn::GenericArgument::Type;


#[derive(Debug, FromField)]
#[darling(attributes(toql))]
pub struct ToqlField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(default)]
    pub column: Option<String>,
     #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub count_query: bool,
    #[darling(default)]
    pub select_always: bool,
    #[darling(default)]
    pub ignore_wildcard: bool,
    #[darling(default)]
    pub field: Option<String>,
    #[darling(default)]
    pub sql: Option<String>,
    #[darling(multiple)]
    pub role: Vec<String>,
    #[darling(default)]
    pub join: Option<String>,
    #[darling(default)]
    pub merge: Option<String>,
    #[darling(default)]
    pub alias: Option<String>,
    #[darling(default)]
    pub table: Option<String> // Alternative sql table name
}



impl ToqlField {
   

  pub fn first_type<'a>(&'a self)-> &'a Ident {
      let types= self.get_types();
      types.0
  }
  pub fn first_non_generic_type<'a>(&'a self)-> Option<&'a Ident> {

      let types= self.get_types();
        if types.2.is_some() {
            types.2
        }
        else if types.1.is_some() {
            types.1
        } else  {
        Some(types.0)
        }
    }



    pub fn get_types<'a>(&'a self) -> (&'a syn::Ident, Option<&'a syn::Ident>, Option<&'a syn::Ident> ){ 
        let type_ident=  Self::get_type(&self.ty).expect(&format!("Invalid type on field {:?}", self.field));

         match &self.ty {
             syn::Type::Path(syn::TypePath{qself:_, path}) => {
                    match &path.segments[0].arguments {
                         syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { colon2_token:_, lt_token:_, gt_token:_, args}) =>{
                             match &args[0] {
                                 Type(t) =>{ 
                                      let gt = match &t {
                                            syn::Type::Path(syn::TypePath{qself:_, path}) => {
                                            match &path.segments[0].arguments {
                                                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { colon2_token:_, lt_token:_, gt_token:_, args}) =>{
                                                    match &args[0] {
                                                        Type(t) =>  Self::get_type(t),
                                                        _ => None
                                                            
                                                    }
                                                },
                                                _ => None
                                            }
                                            },
                                            _=> None
                                      };
                                     ( type_ident,Self::get_type(t), gt)},
                                  _ => (type_ident, None, None)
                             }
                            
                         },  
                         _ => (type_ident, None, None)
                    }
             },
             _ => (type_ident, None, None)
            }
         }

     fn get_type<'a>(ty: &'a  syn::Type) -> Option<&'a syn::Ident> {
         match ty {
             syn::Type::Path(syn::TypePath{qself:_, path}) => {
               Some(&path.segments[0].ident)
             },
             _ => None
            }
         
    }
 
}


#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tables, columns, alter, alias), forward_attrs(allow, doc, cfg), supports(struct_any))]
pub struct Toql {
    pub ident: syn::Ident,
    pub attrs: Vec<syn::Attribute>,
     #[darling(default)]
    pub tables: Option<String>,
     #[darling(default)]
    pub columns: Option<String>,
      #[darling(default)]
    pub alias: Option<String>,
     #[darling(default)]
    pub alter: Option<String>,
    pub data: darling::ast::Data<(), ToqlField>
}



impl quote::ToTokens for Toql {
    fn to_tokens(&self, tokens: &mut  proc_macro2::TokenStream) {

    //println!("DARLING = {:?}", self);

        let mut gen = GeneratedToql::from_toql(&self);
     
        #[cfg(feature = "mysqldb")]
        let mut mysql = GeneratedMysql::from_toql(&self);
         

         let Toql {
            ident:_,
            attrs:_,
            ref tables,
             ref columns,
             alias:_,
             alter:_,
             ref data,
        } = *self;

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;
        
        for field in fields {
            
                if field.skip {
                    continue;
                }
                let result = gen.add_field_mapping(&self, field);

                // Don't build other code for invalid field, process next field
                if result.is_err() {
                    continue;
                }

                gen.add_field_for_builder(&self, field);

                if field.merge.is_some(){
                    gen.add_merge_function(&self, field);  

                    #[cfg(feature = "mysqldb")]   
                    mysql.add_ignored_path(&self, field);

                    #[cfg(feature = "mysqldb")]
                    mysql.add_path_loader(&self, field);

                    #[cfg(feature = "mysqldb")]
                    mysql.add_merge_predicates(&self, field);
                } 
                
                #[cfg(feature = "mysqldb")]
                mysql.add_mysql_deserialize(&self, field);
                
               
        }

       
      
        tokens.extend(quote!(#gen));
        
        #[cfg(feature = "mysqldb")]
        tokens.extend(quote!(#mysql));
          
    }
}