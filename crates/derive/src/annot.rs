use crate::toql::codegen_api::CodegenApi;
use crate::toql::codegen_update::CodegenUpdate;
use crate::toql::codegen_key::CodegenKey;
use crate::toql::codegen_tree::CodegenTree;
use crate::toql::codegen_mapper::CodegenMapper;
use crate::toql::codegen_query_fields::CodegenQueryFields;
use crate::toql::codegen_key_from_row::CodegenKeyFromRow;
use crate::toql::codegen_entity_from_row::CodegenEntityFromRow;
use crate::toql::codegen_insert::CodegenInsert;
use crate::string_set::StringSet;


use syn::Path;

#[derive(Debug, FromMeta, Clone)]
pub struct Pair {
    #[darling(rename = "self")]
    pub this: String,
    pub other: String,
}
#[derive(Debug, FromMeta, Default, Clone)]
pub struct FieldRoles {
     #[darling(default)]
    pub load: Option<String>,
     #[darling(default)]
    pub update: Option<String>
}
#[derive(Debug, FromMeta, Default, Clone)]
pub struct StructRoles {
    #[darling(default)]
    pub load: Option<String>,
    #[darling(default)]
    pub update: Option<String>,
     #[darling(default)]
    pub insert: Option<String>,
     #[darling(default)]
    pub delete: Option<String>
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
    #[darling(default)]
    pub merge: Option<MergeArg>,
    #[darling(default)]
    pub alias: Option<String>,
   /*  #[darling(default)]
    pub table: Option<String>, // Alternative sql table name */
    #[darling(default)]
    pub handler: Option<Path>,
    #[darling(multiple)]
    pub param: Vec<ParamArg>,
    #[darling(default)]
    pub roles: FieldRoles,
    #[darling(multiple)]
    pub on_param: Vec<OnParamArg>
            
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
pub struct SelectionArg {
    pub name: String,
    pub fields: String,
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
   
    #[darling(multiple)]
    pub predicate: Vec<PredicateArg>,
    #[darling(multiple)]
    pub selection: Vec<SelectionArg>,
     #[darling(default)]
    pub roles : StructRoles,
    /* 
    #[darling(multiple)]
    pub insdel_role: Vec<String>,
    #[darling(multiple)]
    pub upd_role: Vec<String>, */
   
    #[darling(default)]
    pub wildcard: Option<StringSet>,

  /*   #[darling(default)]
    pub count_filter: Option<StringSet>, */

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
        let toql_api = CodegenApi::from_toql(&rust_struct);
    

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
            auto_key: _,
            skip_select: _,
            skip_query_builder,
            predicate: _,
            selection: _,
            roles: _,
            wildcard:_,
           // count_filter:_,
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
                        toql_entity_from_row.add_deserialize_skip_field(&f);
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

                       
                        toql_entity_from_row.add_ignored_path(&f);

                       
                        toql_entity_from_row.add_path_loader(&f);
                    }

                    
                    toql_entity_from_row.add_deserialize(&f);

                   

                   
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
                 
                }
              
            }

            // Fail if no keys are found
            if toql_key.key_missing() {
                return Err(darling::Error::custom(
                    "No field(s) marked as key. Add `key` to #[toql(...)] ",
                ));
            }

            // Build merge functionality
          
            toql_entity_from_row.build_merge();

            Ok(())
        };

        match build_fields() {
            Result::Err(err) => {
                tokens.extend(err.write_errors());
            }
            _ => {
                // Produce compiler tokens
                tokens.extend(quote!(#toql_api));
                tokens.extend(quote!(#toql_key));
                tokens.extend(quote!(#toql_tree));

                tokens.extend(quote!(#toql_key_from_row));
                tokens.extend(quote!(#toql_entity_from_row));

                if !skip_query_builder {
                    tokens.extend(quote!(#toql_query_fields));
                }

                if !skip_load {
                    tokens.extend(quote!(#toql_mapper));
                   
                }

                if !skip_mut {
                    tokens.extend(quote!(#toql_insert));
                    tokens.extend(quote!(#toql_update));
                  
                }

            }
        }
    }
}
