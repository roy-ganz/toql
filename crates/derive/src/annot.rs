use crate::toql::codegen_update::CodegenUpdate;
use crate::toql::codegen_key::CodegenKey;
use crate::toql::codegen_tree::CodegenTree;
use crate::toql::codegen_mapper::CodegenMapper;
use crate::toql::codegen_query_fields::CodegenQueryFields;
use crate::toql::codegen_key_from_row::CodegenKeyFromRow;
use crate::toql::codegen_entity_from_row::CodegenEntityFromRow;
use crate::toql::codegen_insert::CodegenInsert;
use crate::string_set::StringSet;

use syn::GenericArgument::Type;
use syn::{Ident, Path};

#[derive(Debug, FromMeta, Clone)]
pub struct Pair {
    #[darling(rename = "self")]
    pub this: String,
    pub other: String,
}

#[derive(Debug, FromMeta)]
pub struct MergeArg {
    /*  #[darling(default, multiple)]
    pub fields: Vec<Pair>, */
    #[darling(default, multiple)]
    pub columns: Vec<Pair>,
    /*
    #[darling(default)]
    pub on_sql: Option<String>, */
    #[darling(default)]
    pub join_sql: Option<String>,

    #[darling(default)]
    pub on_sql: Option<String>,
}

#[derive(Debug, FromMeta)]
pub struct JoinArg {
    #[darling(default, multiple)]
    pub columns: Vec<Pair>,

    #[darling(default)]
    pub on_sql: Option<String>,
    
     #[darling(default)]
    pub join_sql: Option<String>, 

    //#[darling(default)]
    //pub discr_sql: Option<String>,
}


// Attribute on struct field
#[derive(Debug, FromField)]
#[darling(attributes(toql))]
pub struct ToqlField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(default)]
    pub join: Option<JoinArg>,
     
    #[darling(default)]
    pub column: Option<String>,
    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub skip_mut: bool,
    #[darling(default)]
    pub skip_query: bool,
   /*  #[darling(default)]
    pub count_filter: bool, */
    #[darling(default)]
    pub count_select: bool,
    #[darling(default)]
    pub preselect: bool,
    #[darling(default)]
    pub skip_wildcard: bool,
    #[darling(default)]
    pub key: bool,
    #[darling(default)]
    pub field: Option<String>,
    #[darling(default)]
    pub sql: Option<String>,
    #[darling(multiple)]
    pub load_role: Vec<String>,
    #[darling(multiple)]
    pub upd_role: Vec<String>,
    #[darling(default)]
    pub merge: Option<MergeArg>,
    #[darling(default)]
    pub alias: Option<String>,
    #[darling(default)]
    pub table: Option<String>, // Alternative sql table name
    #[darling(default)]
    pub handler: Option<Path>,
    #[darling(multiple)]
    pub param: Vec<ParamArg>,

    #[darling(multiple)]
    pub on_param: Vec<OnParamArg>,
            
}



 


impl ToqlField {
    // IMPROVE: Function is used, but somehow considered unused
  /*   #[allow(dead_code)]
    pub fn _first_type<'a>(&'a self) -> &'a Ident {
        let types = self.get_types();
        types.0
    } */
    /* pub fn first_non_generic_type<'a>(&'a self) -> Option<&'a Ident> {

        

        let types = self.get_types();
        if types.2.is_some() {
            types.2
        } else if types.1.is_some() {
            types.1
        } else {
            Some(types.0)
        }
    }
    pub fn number_of_options<'a>(&'a self) -> u8 {
        let types = self.get_types();

        let mut n: u8 = 0;
        if types.0 == "Option" {
            n += 1;
            if let Some(t) = types.1 {
                if t == "Option" {
                    n += 1;
                    if let Some(t) = types.2 {
                        if t == "Option" {
                            n += 1;
                        }
                    }
                }
            }
        }
        n
    }

    pub fn get_types<'a>(
        &'a self,
    ) -> (
        &'a syn::Ident,
        Option<&'a syn::Ident>,
        Option<&'a syn::Ident>,
    ) {

        // Test
        println!("PARSE TEST");
        dbg!(TypeInfo::parse(&self.ty));

        let type_ident =
            Self::get_type(&self.ty).expect(&format!("Invalid type on field {:?}", self.field));

        match &self.ty {
            syn::Type::Path(syn::TypePath { qself: _, path }) => {
                match &path.segments[0].arguments {
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        gt_token: _,
                        args,
                    }) => match &args[0] {
                        Type(t) => {
                            let gt = match &t {
                                syn::Type::Path(syn::TypePath { qself: _, path }) => {
                                    match &path.segments[0].arguments {
                                        syn::PathArguments::AngleBracketed(
                                            syn::AngleBracketedGenericArguments {
                                                colon2_token: _,
                                                lt_token: _,
                                                gt_token: _,
                                                args,
                                            },
                                        ) => match &args[0] {
                                            Type(t) => Self::get_type(t),
                                            _ => None,
                                        },
                                        _ => None,
                                    }
                                }
                                _ => None,
                            };
                            (type_ident, Self::get_type(t), gt)
                        }
                        _ => (type_ident, None, None),
                    },
                    _ => (type_ident, None, None),
                }
            }
            _ => (type_ident, None, None),
        }
    }

    fn get_type<'a>(ty: &'a syn::Type) -> Option<&'a syn::Ident> {
        match ty {
            syn::Type::Path(syn::TypePath { qself: _, path }) => Some(&path.segments[0].ident),
            _ => None,
        }
    } */
}

#[derive(FromMeta, PartialEq, Eq, Clone, Debug)]
pub enum RenameCase {
    #[darling(rename = "CamelCase")]
    CamelCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "SHOUTY_SNAKE_CASE")]
    ShoutySnakeCase,
    #[darling(rename = "mixedCase")]
    MixedCase,
}
#[derive(FromMeta, Clone, Debug)]
pub struct PredicateArg {
    pub name: String,
    pub sql: String,

    #[darling(default)]
    pub handler: Option<Path>,

    #[darling(multiple)]
    pub on_param: Vec<PredicateOnParamArg>,

    #[darling(default)]
    pub count_filter: bool
}

#[derive(FromMeta, Clone, Debug)]
pub struct OnParamArg {
    pub name: String,
}
#[derive(FromMeta, Clone, Debug)]
pub struct PredicateOnParamArg {
    pub name: String,
    #[darling(default)]
    pub index: u8,
}
#[derive(FromMeta, Clone, Debug)]
pub struct ParamArg {
    pub name: String,
    #[darling(default)]
    pub value: String,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(toql), forward_attrs(allow, doc, cfg), supports(struct_any))]
pub struct Toql {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub attrs: Vec<syn::Attribute>,
    #[darling(default)]
    pub tables: Option<RenameCase>,
    #[darling(default)]
    pub table: Option<String>,
    #[darling(default)]
    pub columns: Option<RenameCase>,
    #[darling(default)]
    pub alias: Option<String>,
    #[darling(default)]
    pub skip_mut: bool,
    #[darling(default)]
    pub skip_load: bool,
    #[darling(default)]
    pub auto_key: bool,
    #[darling(default)]
    pub skip_select: bool,
    #[darling(default)]
    pub skip_query_builder: bool,
    #[darling(default)]
    pub serde_key: bool,
    #[darling(multiple)]
    pub predicate: Vec<PredicateArg>,
    #[darling(multiple)]
    pub insdel_role: Vec<String>,
    #[darling(multiple)]
    pub upd_role: Vec<String>,
   
    #[darling(default)]
    pub wildcard: Option<StringSet>,

    #[darling(default)]
    pub count_filter: Option<StringSet>,

    pub data: darling::ast::Data<(), ToqlField>,
}

impl quote::ToTokens for Toql {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        //println!("DARLING = {:?}", self);

        let rust_struct = crate::sane::Struct::create(&self);
        let mut toql_mapper = CodegenMapper::from_toql(&rust_struct);
        let mut toql_query_fields = CodegenQueryFields::from_toql(&rust_struct);
        let mut toql_update = CodegenUpdate::from_toql(&rust_struct);
        let mut toql_key = CodegenKey::from_toql(&rust_struct);
        let mut toql_tree = CodegenTree::from_toql(&rust_struct);
        let mut toql_key_from_row = CodegenKeyFromRow::from_toql(&rust_struct);
        let mut toql_entity_from_row = CodegenEntityFromRow::from_toql(&rust_struct);
        let mut toql_insert = CodegenInsert::from_toql(&rust_struct);

       /*  #[cfg(feature = "mysql15")]
        let mut mysql15_load = crate::mysql15::codegen_load::GeneratedMysqlLoad::from_toql(&rust_struct);

           #[cfg(feature = "mysql15")]
        let mut mysql15_tree = crate::mysql15::codegen_tree::GeneratedMysqlTree::from_toql(&rust_struct);
 */

        #[cfg(feature = "mysql15")]
        let mut mysql15_insert = crate::mysql15::codegen_insert::GeneratedMysqlInsert::from_toql(&rust_struct);

        #[cfg(feature = "mysql15")]
        let mut mysql15_macros = crate::mysql15::codegen_macros::CodegenMacros::from_toql(&rust_struct);

        let Toql {
            vis: _,
            ident: _,
            attrs: _,
            tables: _,
            table: _,
            columns: _,
            alias: _,
            skip_mut,
            skip_load,
            auto_key,
            skip_select,
            skip_query_builder,
            serde_key: _,
            predicate: _,
            insdel_role: _,
            upd_role: _,
            wildcard:_,
            count_filter:_,
            ref data,
        } = *self;

       

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let build_fields = || -> darling::error::Result<()> {
            for field in fields {
                let f = crate::sane::Field::create(&field, &self)?;

                toql_key.add_key_field(&f)?;

                // Generate query functionality
                if !skip_load {
                    if field.skip {
                        #[cfg(feature = "mysql15")]
                        toql_entity_from_row.add_mysql_deserialize_skip_field(&f);
                        continue;
                    }
                    toql_mapper.add_field_mapping(&f)?;

                    // Don't build further code for invalid field, process next field
                   /*  if result.is_err() {
                        continue;
                    } */

                    toql_query_fields.add_field_for_builder(&f);

                    toql_tree.add_tree_traits(&f);

                    toql_key_from_row.add_key_deserialize(&f)?;

                  
                    if field.merge.is_some() {
                        toql_mapper.add_merge_function(&f);

                        #[cfg(feature = "mysql15")]
                        toql_entity_from_row.add_ignored_path(&f);

                        #[cfg(feature = "mysql15")]
                        toql_entity_from_row.add_path_loader(&f);
                    }

                    #[cfg(feature = "mysql15")]
                    toql_entity_from_row.add_mysql_deserialize(&f);

                   

                   
                   /*  if result.is_err() {
                        // tokens.extend(result.err());
                        continue;
                    } */
                }

                // Generate insert/delete/update functionality
                // Select is considered part of mutation functionality (Copy)
                if !skip_mut {
                    toql_update.add_tree_update(&f);
                
                    toql_insert.add_tree_insert(&f)?;

                   /*  #[cfg(feature = "mysql15")]
                    mysql15_insert.add_insert_field(&f); */

                 
                }
               /*  if !skip_select {
                    #[cfg(feature = "mysql15")]
                    mysql_select.add_select_field(&f)?;
                } */
            }

            // Fail if no keys are found
            if toql_key.key_missing() {
                return Err(darling::Error::custom(
                    "No field(s) marked as key. Add `key` to #[toql(...)] ",
                ));
            }

            // Build merge functionality
            /*  #[cfg(feature = "mysql15")]
            mysql_select.build_merge();  SELECT on signle table only*/
            #[cfg(feature = "mysql15")]
            toql_entity_from_row.build_merge();

            Ok(())
        };

        match build_fields() {
            Result::Err(err) => {
                tokens.extend(err.write_errors());
            }
            _ => {
                // Produce compiler tokens
                tokens.extend(quote!(#toql_key));
                tokens.extend(quote!(#toql_tree));

                tokens.extend(quote!(#toql_key_from_row));
                tokens.extend(quote!(#toql_entity_from_row));

                if !skip_query_builder {
                    tokens.extend(quote!(#toql_query_fields));
                }

                if !skip_load {
                    tokens.extend(quote!(#toql_mapper));

                  /*   #[cfg(feature = "mysql15")]
                    tokens.extend(quote!(#mysql15_load)); */

                    #[cfg(feature = "mysql15")]
                    tokens.extend(quote!(#mysql15_macros));

                  /*   #[cfg(feature = "mysql15")]
                    tokens.extend(quote!(#mysql15_key)); */

                   
                }

                if !skip_mut {
                    tokens.extend(quote!(#toql_insert));
                    tokens.extend(quote!(#toql_update));

                   /*  #[cfg(feature = "mysql15")]
                    tokens.extend(quote!(#mysql15_insert)); */
                  
                }

                /* if !skip_select {
                    #[cfg(feature = "mysql15")]
                    tokens.extend(quote!(#mysql_select));
                } */
            }
        }
    }
}
