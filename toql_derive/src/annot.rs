use crate::codegen_toql_mutate::GeneratedToqlMutate;
use crate::codegen_toql_mapper::GeneratedToqlMapper;
use crate::codegen_toql_query_builder::GeneratedToqlQueryBuilder;

#[cfg(feature = "mysqldb")]
use crate::codegen_mysql_load::GeneratedMysqlLoad;

#[cfg(feature = "mysqldb")]
use crate::codegen_mysql_select::GeneratedMysqlSelect;

use syn::GenericArgument::Type;
use syn::Ident;


#[derive(Debug, FromMeta)]
pub struct Pair {
    #[darling(rename = "self")]
    pub this : String,
    pub other: String
}


#[derive(Debug, FromMeta)]
pub struct MergeArg {
    #[darling(rename = "self_field", default)]
    pub this_field: Option<String>, 
    #[darling(default)]
    pub other_field: Option<String>,
    #[darling(default)]
    pub on_sql: Option<String>,

    #[darling(default, multiple)]
    pub fields: Vec<Pair>
}


#[derive(Debug, FromMeta)]
pub struct JoinArg {
     #[darling(rename = "self_column", multiple)]
    pub this_columns: Vec<String>,

     #[darling(rename = "other_column", multiple)]
    pub other_columns: Vec<String>, 

    #[darling(default, multiple)]
    pub columns: Vec<Pair>,

    #[darling(default)]
    pub on_sql: Option<String>,

}

// Attribute on struct field
#[derive(Debug, FromField)]
#[darling(attributes(toql) )]
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
    pub count_filter: bool,
    #[darling(default)]
    pub count_select: bool,
    #[darling(default)]
    pub preselect: bool,
    #[darling(default)]
    pub ignore_wildcard: bool,
    #[darling(default)]
    pub key: bool,
    #[darling(default)]
    pub field: Option<String>,
    #[darling(default)]
    pub sql: Option<String>,
    #[darling(multiple)]
    pub role: Vec<String>,
    #[darling(default, multiple)]
    pub merge: Vec<MergeArg>,
    #[darling(default)]
    pub alias: Option<String>,
    #[darling(default)]
    pub table: Option<String>, // Alternative sql table name
}

impl ToqlField {


    // IMPROVE: Function is used, but somehow considered unused
    #[allow(dead_code)]
    pub fn _first_type<'a>(&'a self) -> &'a Ident {
        let types = self.get_types();
        types.0
    }
    pub fn first_non_generic_type<'a>(&'a self) -> Option<&'a Ident> {
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
    }
}

#[derive(FromMeta, PartialEq, Eq, Debug)]
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
    pub skip_query: bool,
    #[darling(default)]
    pub skip_query_builder: bool,
    pub data: darling::ast::Data<(), ToqlField>,
}


 

impl quote::ToTokens for Toql {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        //println!("DARLING = {:?}", self);

        let mut toql_mapper = GeneratedToqlMapper::from_toql(&self);
        let mut toql_query_builder = GeneratedToqlQueryBuilder::from_toql(&self);
        let mut toql_mutate = GeneratedToqlMutate::from_toql(&self);

        #[cfg(feature = "mysqldb")]
        let mut mysql_load = GeneratedMysqlLoad::from_toql(&self);

        #[cfg(feature = "mysqldb")]
        let mut mysql_select = GeneratedMysqlSelect::from_toql(&self);

        let Toql {
            vis: _,
            ident: _,
            attrs: _,
            tables: _,
            table: _,
            columns: _,
            alias: _,
            skip_mut,
            skip_query,
            skip_query_builder,
            ref data,
        } = *self;

        let mut_enabled = !skip_mut;
        let query_enabled = !skip_query;
        let query_builder_enabled = !skip_query_builder;

        let mut fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        for field in fields {


            // Generate query functionality
            if query_enabled {
                if field.skip {
                    #[cfg(feature = "mysqldb")]
                    mysql_load.add_mysql_deserialize_skip_field(field);
                    continue;
                }
                let result = toql_mapper.add_field_mapping(&self, field);

                // Don't build further code for invalid field, process next field
                if result.is_err() {
                    continue;
                }

                if query_builder_enabled {
                    toql_query_builder.add_field_for_builder(&self, field);
                }

                if !field.merge.is_empty() {
                    toql_mapper.add_merge_function(&self, field);

                    #[cfg(feature = "mysqldb")]
                    mysql_load.add_ignored_path(&self, field);

                    #[cfg(feature = "mysqldb")]
                    mysql_load.add_path_loader(&self, field);

                    #[cfg(feature = "mysqldb")]
                    mysql_load.add_merge_predicates(&self, field);

                    
                }

                #[cfg(feature = "mysqldb")]
                mysql_load.add_mysql_deserialize(&self, field);

                 #[cfg(feature = "mysqldb")]
                let result = mysql_select.add_select_field(&self, field);
                if result.is_err() {
                   // tokens.extend(result.err());
                    continue;
                }
            }

            // Generate insert/delete/update functionality
            if mut_enabled {
                toql_mutate.add_mutate_field(&self, field);
            }
        }

        // Produce compiler tokens
        if query_builder_enabled {
            tokens.extend(quote!(#toql_query_builder));
        }

        if query_enabled {
            tokens.extend(quote!(#toql_mapper));

            #[cfg(feature = "mysqldb")]
            tokens.extend(quote!(#mysql_load));

            #[cfg(feature = "mysqldb")]
            tokens.extend(quote!(#mysql_select));
        }

        if mut_enabled {
            tokens.extend(quote!(#toql_mutate));
        }
    }
}
