use crate::annot::FieldRoles;
use crate::annot::OnAuxParamArg;
use crate::annot::Pair;
use crate::annot::RenameCase;
use crate::annot::Toql;
use crate::annot::ToqlField;
use crate::annot::{ParamArg, PredicateArg, SelectionArg, StructRoles};
use crate::heck::MixedCase;
use crate::heck::SnakeCase;

use proc_macro2::{Span, TokenStream};
use syn::{Ident, Path, Visibility};

use std::collections::HashSet;

//use crate::error::Result;
use darling::Result;

pub struct Struct {
    pub rust_struct_ident: Ident,
    pub rust_struct_name: String,
    pub sql_table_name: String,
    pub sql_table_alias: String,
    pub rust_struct_visibility: Visibility,
    pub mapped_predicates: Vec<PredicateArg>,
    pub mapped_selections: Vec<SelectionArg>,
    pub roles: StructRoles,
    pub wildcard: Option<HashSet<String>>,
    pub auto_key: bool,
    pub skip_mut: bool,
}

impl Struct {
    pub fn create(toql: &Toql) -> Self {
        let renamed_table = crate::util::rename_or_default(&toql.ident.to_string(), &toql.tables);

        let mapped_predicates: Vec<PredicateArg> = toql
            .predicate
            .iter()
            .map(|a| PredicateArg {
                name: a.name.to_mixed_case(),
                sql: a.sql.clone(),
                handler: a.handler.clone(),
                on_aux_param: a.on_aux_param.clone(),
                count_filter: a.count_filter,
            })
            .collect::<Vec<_>>();
        let mapped_selections: Vec<SelectionArg> = toql
            .selection
            .iter()
            .map(|a| SelectionArg {
                name: if a.name.is_empty() {
                    "std".to_string()
                } else {
                    a.name.to_mixed_case()
                },
                fields: a.fields.clone(),
            })
            .collect::<Vec<_>>();

        Struct {
            rust_struct_ident: toql.ident.clone(),
            rust_struct_name: toql.ident.to_string(),
            sql_table_name: toql.table.clone().unwrap_or(renamed_table),
            sql_table_alias: toql
                .alias
                .clone()
                .unwrap_or_else(|| toql.ident.to_string())
                .to_mixed_case(),
            rust_struct_visibility: toql.vis.clone(),
            mapped_predicates,
            mapped_selections,
            roles: toql.roles.clone(),
            wildcard: toql.wildcard.as_ref().map(|e| e.0.to_owned()), //.as_ref().map(|v| v.split(",").map(|s| s.trim().to_string()).collect::<HashSet<String>>()).to_owned(),
            //  count_filter: toql.count_filter.as_ref().map(|e|e.0.to_owned()), //Some(toql.count_filter.0); //toql.count_filter.as_ref().map(|v| v.split(",").map(|s| s.trim().to_string()).collect::<HashSet<String>>()).to_owned()
            auto_key: toql.auto_key,
            skip_mut: toql.skip_mut,
        }
    }
}

#[derive(Clone)]
pub enum SqlTarget {
    Column(String),
    Expression(String),
}
// Maybe fillup this
#[derive(Clone)]
pub struct RegularField {
    pub sql_target: SqlTarget,
    pub key: bool,
    //  pub count_select: bool,
    //  pub count_filter: bool,
    pub handler: Option<Path>,
    pub default_inverse_column: Option<String>,
    pub aux_params: Vec<ParamArg>,
    //pub on_aux_params: Vec<OnAuxParamArg>,
    pub foreign_key: bool, // Column of this field is used as foreign key
}
#[derive(Clone)]
pub struct JoinField {
    // pub sql_join_table_ident: Ident,
    pub sql_join_table_name: String,
    pub join_alias: String,
    pub default_self_column_code: TokenStream,
    pub columns_map_code: TokenStream,
    pub translated_default_self_column_code: TokenStream,
    pub translated_columns_map_code: TokenStream,
    pub on_sql: Option<String>,
    pub key: bool,
    pub aux_params: Vec<ParamArg>,
    pub columns: Vec<Pair>,
    pub partial_table: bool,
    pub foreign_key: bool, // Column(s) of this join key is used as foreign key
}

#[derive(Clone)]
pub enum MergeColumn {
    Aliased(String),
    Unaliased(String),
}
#[derive(Clone)]
pub struct MergeMatch {
    pub other: MergeColumn,
    pub this: String,
}

#[derive(Clone)]
pub struct MergeField {
    // pub columns: RenameCase,
    //   pub sql_join_table_ident: Ident,
    pub sql_join_table_name: String,
    pub join_alias: String,

    pub columns: Vec<MergeMatch>,
    pub join_sql: Option<String>,
    pub on_sql: Option<String>,
}

impl MergeField {
    /*  pub fn column(&self, field_name: &str) -> String {
        crate::util::rename(&field_name, &self.columns)
    } */

    /* pub fn other_field(&self, this_field: &str, default_other_field: String) -> String {
        // Lookup field renaming
        let other_field = self
            .cols
            .iter()
            .find(|&f| &f.this == this_field)
            .map_or(default_other_field, |p| String::from(p.other.as_str()));

        other_field
    } */
}

#[derive(Clone)]
pub struct Field {
    pub rust_field_ident: Ident,
    pub rust_field_name: String,
    pub rust_type_ident: syn::Type,
    pub rust_base_type_ident: syn::Ident,
    pub rust_type_name: String,
    pub toql_field_name: String,
    pub number_of_options: u8,
    pub skip_wildcard: bool,
    pub roles: FieldRoles,
    pub preselect: bool,
    pub kind: FieldKind,
    pub skip_mut: bool,
    //   pub skip_query: bool,
}

#[derive(Clone)]
pub enum FieldKind {
    Regular(RegularField),
    Join(JoinField),
    Merge(MergeField),
}

impl Field {
    pub fn create(field: &ToqlField, toql: &Toql) -> Result<Self> {
        let mut number_of_options = 0;
        let rust_field_type = unwrap_type(&field.ty, &mut number_of_options);

        let rust_field_ident = field.ident.as_ref().unwrap().to_owned();
        let rust_field_name = rust_field_ident.to_string();
        let rust_type_ident = rust_field_type.to_owned();

        let rust_base_type_ident = unwrap_base(&rust_field_type);
        let rust_type_name = rust_base_type_ident.to_string();

        let toql_field_name = rust_field_name.trim_start_matches("r#").to_mixed_case();

        if field.skip_wildcard && field.preselect {
            return Err(darling::Error::custom(
                "`skip_wildcard` is not allowed together with `preselect`. Change `#[toql(..)]`."
                    .to_string(),
            )
            .with_span(&field.ident));
        }
        if field.skip_wildcard && number_of_options == 0 {
            return Err(darling::Error::custom(
                    "`skip_wildcard` is only allowed on selectable fields. Add `Option<..>` to field type.".to_string(),
                )
                .with_span(&field.ident));
        }

        let kind = if field.join.is_some() {
            if field.handler.is_some() {
                return Err(darling::Error::custom(
                    "`handler` not allowed for joined fields. Remove from `#[toql(..)]`."
                        .to_string(),
                )
                .with_span(&field.ident));
            }
            if field.sql.is_some() {
                return Err(darling::Error::custom(
                    "`sql` not allowed for joined fields. Remove from `#[toql(..)]`.".to_string(),
                )
                .with_span(&field.ident));
            }

            if field.key && field.join.as_ref().unwrap().partial_table {
                return Err(darling::Error::custom(
                    "`partial table` not allowed for key fields. Remove from `#[toql(..)]`."
                        .to_string(),
                )
                .with_span(&field.ident));
            }

            let sql_join_table_name = crate::util::rename_or_default(
                &rust_type_name, // field.first_non_generic_type().unwrap().to_string().as_str(),
                &toql.tables,
            );
            //  let sql_join_table_name = field.table.as_ref().unwrap_or(&renamed_table).to_owned();
            //  let sql_join_table_name = renamed_table.to_owned();
            let columns_translation = field
                .join
                .as_ref()
                .unwrap()
                .columns
                .iter()
                .map(|column| {
                    let tc = &column.this;
                    let oc = &column.other;
                    quote!(#oc => #tc, )
                })
                .collect::<Vec<_>>();

            let translated_columns_translation = field
                .join
                .as_ref()
                .unwrap()
                .columns
                .iter()
                .map(|column| {
                    let tc = &column.this;
                    let oc = &column.other;
                    quote!(#oc => mapper.translate_aliased_column(sql_alias,#tc), )
                })
                .collect::<Vec<_>>();

            let other_columns: Vec<String> = field
                .join
                .as_ref()
                .unwrap()
                .columns
                .iter()
                .map(|column| String::from(column.other.as_str()))
                .collect::<Vec<_>>();

            let default_self_column_format = format!("{}_{{}}", field.ident.as_ref().unwrap());
            let default_self_column_code = quote!( let default_self_column= format!(#default_self_column_format, other_column););

            let translated_default_self_column_code = quote!( let default_self_column= mapper.translate_aliased_column(sql_alias, other_column););

            let safety_check_for_column_mapping = if other_columns.is_empty() {
                quote!()
            } else {
                quote!(
                    if cfg!(debug_assertions) {
                        let valid_columns = <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                        let invalid_columns: Vec<String> = [ #(#other_columns),* ]
                            .iter()
                            .filter(|col| !valid_columns.iter().any ( |s| &s == col ) )
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>();

                        if !invalid_columns.is_empty() {
                            /* let valid_columns: Vec<String> = valid_columns
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>(); */
                        toql::tracing::warn!("On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`", #rust_type_name, #rust_field_name, invalid_columns.join(","),valid_columns.join(","));
                        }
                    }
                )
            };

            let columns_map_code = quote!( {

                #safety_check_for_column_mapping

                let self_column = match other_column.as_str(){
                        #(#columns_translation)*
                        _ => &default_self_column
                };
                self_column
            });

            let translated_columns_map_code = quote!( {

                #safety_check_for_column_mapping

                let self_column = match other_column.as_str(){
                        #(#translated_columns_translation)*
                        _ => default_self_column
                };
                self_column
            });

            FieldKind::Join(JoinField {
                //   sql_join_table_ident: Ident::new(&sql_join_table_name, Span::call_site()),
                join_alias: field
                    .alias
                    .as_ref()
                    .unwrap_or(&rust_field_name)
                    .to_mixed_case(),
                sql_join_table_name,
                default_self_column_code,
                columns_map_code,
                translated_default_self_column_code,
                translated_columns_map_code,
                on_sql: field.join.as_ref().unwrap().on_sql.clone(),
                key: field.key,
                aux_params: field.aux_param.clone(),
                columns: field.join.as_ref().unwrap().columns.clone(),
                partial_table: field.join.as_ref().unwrap().partial_table,
                foreign_key: field.foreign_key,
            })
        } else if field.merge.is_some() {
            if field.key {
                return Err(darling::Error::custom(
                    "`key` not allowed for merged fields. Remove from `#[toql(..)]`.".to_string(),
                )
                .with_span(&field.ident));
            }
            if field.handler.is_some() {
                return Err(darling::Error::custom(
                    "`handler` not allowed for merged fields. Remove from `#[toql(..)]`."
                        .to_string(),
                )
                .with_span(&field.ident));
            }
            if field.sql.is_some() {
                return Err(darling::Error::custom(
                    "`sql` not allowed for merged fields. Remove from `#[toql(..)]`.".to_string(),
                )
                .with_span(&field.ident));
            }

            let sql_join_table_name = crate::util::rename_or_default(
                &rust_type_name, //field.first_non_generic_type().unwrap().to_string().as_str(),
                &toql.tables,
            );
            //let sql_join_table_name = field.table.as_ref().unwrap_or(&renamed_table).to_owned();
            // let sql_join_table_name = renamed_table.to_owned();

            let mut columns: Vec<MergeMatch> = Vec::new();

            for m in &field.merge.as_ref().unwrap().columns {
                /*  let parts = m.key.split(".").collect::<Vec<&str>>();

                let key =  match parts.len()  {
                     1 =>  MergeKey::Field(parts.get(0).unwrap() .to_string()),
                     2 =>  MergeKey::Join(parts.get(0).unwrap() .to_string(), parts.get(1).unwrap().to_string()),
                     _ => {
                         return Err(darling::Error::custom(
                             "`key` can only reference field only immediate joined key."
                                 .to_string(),
                         )
                         .with_span(&field.ident));
                     }
                 }; */
                let other = match m.other.find('.').unwrap_or(0) {
                    0 => MergeColumn::Unaliased(m.other.to_string()),
                    _ => MergeColumn::Aliased(m.other.to_string()),
                };

                columns.push(MergeMatch {
                    other,
                    this: m.this.clone(),
                });
            }

            if let Some(j) = field.merge.as_ref().unwrap().join_sql.as_ref() {
                // Search for .., ignore ...
                let mut n = 0;
                let found_self_alias = j.chars().any(|c| {
                    if c == '.' {
                        n += 1;
                        false
                    } else if n == 2 {
                        true
                    } else {
                        n = 0;
                        false
                    }
                });
                if found_self_alias {
                    return Err(darling::Error::custom(
                                "Alias `..` not allowed for merged fields. Use `...` to refer to table of merged entities. Change `#[toql(..)]`.".to_string(),
                            )
                        .with_span(&field.ident));
                }
            }

            FieldKind::Merge(MergeField {
                //   sql_join_table_ident: Ident::new(&sql_join_table_name, Span::call_site()),
                join_alias: field
                    .alias
                    .as_ref()
                    .unwrap_or(&sql_join_table_name.to_snake_case())
                    .to_owned(),
                sql_join_table_name,
                join_sql: field.merge.as_ref().unwrap().join_sql.to_owned(),
                on_sql: field.merge.as_ref().unwrap().on_sql.to_owned(),
                /*  columns: toql
                .columns
                .as_ref()
                .unwrap_or(&RenameCase::SnakeCase)
                .to_owned(), */
                columns,
            })
        } else {
            FieldKind::Regular(RegularField {
                sql_target: if field.sql.is_some() {
                    SqlTarget::Expression(field.sql.as_ref().unwrap().to_owned())
                } else {
                    SqlTarget::Column(match &field.column {
                        Some(string) => string.to_owned(),
                        None => crate::util::rename_or_default(&rust_field_name, &toql.columns),
                    })
                },
                default_inverse_column: if field.sql.is_some() {
                    None
                } else {
                    // TODO put table renaming into separate function

                    let table_name = toql.table.clone().unwrap_or_else(|| toql.ident.to_string());
                    Some(crate::util::rename(
                        &format!("{}_{}", &table_name, &rust_field_name),
                        toql.columns.as_ref().unwrap_or(&RenameCase::SnakeCase),
                    ))
                },
                key: field.key,
                //   count_select: field.count_select,
                //count_filter: field.count_filter,
                handler: field.handler.to_owned(),
                aux_params: field.aux_param.clone(),
                //  on_aux_params: field.on_aux_param.clone(),
                foreign_key: field.foreign_key,
            })
        };

        Ok(Field {
            rust_field_ident,
            rust_field_name,
            rust_type_ident,
            rust_type_name,
            rust_base_type_ident,
            toql_field_name,
            number_of_options,
            skip_mut: field.skip_mut,
            //  skip_query: field.skip_query,
            skip_wildcard: field.skip_wildcard,
            roles: field.roles.clone(),
            preselect: field.preselect,
            kind,
        })
    }
}

pub(crate) fn unwrap_type<'a>(ty: &'a syn::Type, number_of_options: &mut u8) -> &'a syn::Type {
    match ty {
        syn::Type::Path(type_path) if type_path.qself.is_none() => {
            let path_segment = &type_path.path.segments.iter().next().unwrap();
            if &path_segment.ident == "Option" {
                let path_arguments = &path_segment.arguments;
                if let syn::PathArguments::AngleBracketed(params) = path_arguments {
                    if let syn::GenericArgument::Type(ty) = params.args.iter().next().unwrap() {
                        *number_of_options += 1;
                        return unwrap_type(ty, number_of_options);
                    }
                }
            }
        }

        _ => {}
    }

    ty
}

pub(crate) fn unwrap_base(ty: &syn::Type) -> syn::Ident {
    match ty {
        syn::Type::Path(type_path) if type_path.qself.is_none() => {
            let path_segment = &type_path.path.segments.iter().next().unwrap();
            let path_arguments = &path_segment.arguments;

            if let syn::PathArguments::AngleBracketed(params) = path_arguments {
                if let syn::GenericArgument::Type(ty) = params.args.iter().next().unwrap() {
                    return unwrap_base(ty);
                }
            }
            return path_segment.ident.to_owned();
        }
        _ => {}
    }

    syn::Ident::new("Unknown", Span::call_site())
}
