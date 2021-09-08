/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, SqlTarget};
use proc_macro2::TokenStream;

use syn::Ident;

pub(crate) struct CodegenInsert<'a> {
    struct_ident: &'a Ident,
    auto_key: bool,

    duplicate: bool,

    dispatch_columns_code: Vec<TokenStream>,
    dispatch_values_code: Vec<TokenStream>,
    insert_columns_code: Vec<TokenStream>,
    insert_values_code: Vec<TokenStream>,
    struct_ins_roles: &'a Option<String>,
}

impl<'a> CodegenInsert<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenInsert {
        CodegenInsert {
            struct_ident: &toql.rust_struct_ident,
            auto_key: toql.auto_key.to_owned(),

            duplicate: false,

            dispatch_columns_code: Vec::new(),
            dispatch_values_code: Vec::new(),
            insert_columns_code: Vec::new(),
            insert_values_code: Vec::new(),
            struct_ins_roles: &toql.roles.insert,
        }
    }

    pub(crate) fn add_tree_insert(
        &mut self,
        field: &crate::sane::Field,
    ) -> darling::error::Result<()> {
        let rust_field_ident = &field.rust_field_ident;

        let rust_type_ident = &field.rust_type_ident;
        let toql_field_name = &field.toql_field_name;

        // Handle key predicate and parameters

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if regular_attrs.key && self.auto_key {
                    return Ok(());
                }

                if !regular_attrs.key && field.skip_mut {
                    return Ok(());
                }
                if regular_attrs.key && field.skip_mut {
                    return Err(darling::Error::custom(
                                "Key must not be `skip_mut`. Use `#[toql(auto_key=true)]` on your struct, if your key is an auto value.".to_string(),
                            )
                        .with_span(&field.rust_field_ident));
                }

                match regular_attrs.sql_target {
                    SqlTarget::Column(ref sql_column) => self.insert_columns_code.push(quote!(
                                e.push_literal(#sql_column);
                                e.push_literal(", ");
                    )),
                    SqlTarget::Expression(_) => {
                        return Ok(());
                    }
                }

                self.insert_values_code.push( match field.number_of_options {
                    2 => {
                        // Option<Option<T>> (toql selectable of nullable column)
                        quote!(
                             if  let Some(field) = &self . #rust_field_ident  {
                                 values.push_arg(toql::sql_arg::SqlArg::from(field.as_ref()));
                                 values.push_literal(", ");
                             } else {
                                values.push_literal("DEFAULT, ");
                             }
                        )
                    }
                    1 if field.preselect => {
                        // Option<T>  selected (nullable column)
                        quote!(
                              values.push_arg( toql::sql_arg::SqlArg::from(self . #rust_field_ident.as_ref()));
                              values.push_literal(", ");
                        )
                    }
                    1 if !field.preselect => {
                        // Option<T>  (toql selectable)
                        quote!(
                            if  let Some(field) = &self . #rust_field_ident {
                                 values.push_arg( toql::sql_arg::SqlArg::from(field));
                                   values.push_literal(", ");
                            } else {
                                  values.push_literal("DEFAULT, ");
                            }
                        )
                    }
                    _ => {
                        // selected field
                        quote!(
                            values.push_arg(toql::sql_arg::SqlArg::from(&self . #rust_field_ident));
                            values.push_literal(", ");
                        )
                    }
                });

                // Structs with keys that are insertable may have duplicates
                // Implement marker trait for them
                if regular_attrs.key && !field.skip_mut {
                    self.duplicate = true;
                }

                if regular_attrs.key && field.skip_mut {
                    self.duplicate = true;
                }
            }

            FieldKind::Join(join_attrs) => {
                if join_attrs.key && self.auto_key {
                    return Ok(());
                }

                if !join_attrs.key && field.skip_mut {
                    return Ok(());
                }
                if join_attrs.key && field.skip_mut {
                    return Err(darling::Error::custom(
                                "Key must not be `skip_mut`. Use `#[toql(auto_key=true)]` on your struct, if your key is an auto value.".to_string(),
                            )
                        .with_span(&field.rust_field_ident));
                }

                self.dispatch_columns_code.push(quote!(
                        #toql_field_name => {
                            return Ok(<#rust_type_ident as toql::tree::tree_insert::TreeInsert>::
                            columns(&mut descendents)?);
                        }
                ));

                self.dispatch_values_code.push(
                   match field.number_of_options  {
                                2 => {quote!(
                                    #toql_field_name => {
                                         if let Some(f) = &mut self. #rust_field_ident .as_ref() {
                                              if let Some(f) = f .as_ref() {
                                                <#rust_type_ident as toql::tree::tree_insert::TreeInsert>::
                                                values(f, &mut descendents, roles, selected_keys, values)?
                                            }
                                         }
                                    }
                                        ) },
                                1 => {quote!(
                                    #toql_field_name => {
                                        if let Some(f) = &mut self. #rust_field_ident .as_ref() {
                                            <#rust_type_ident as toql::tree::tree_insert::TreeInsert>::
                                            values(f, &mut descendents, roles, selected_keys, values)?
                                        }
                                    }
                                        ) },
                                _ => {
                                    quote!(
                                        #toql_field_name => {
                                            <#rust_type_ident as toql::tree::tree_insert::TreeInsert>::
                                            values(& self. #rust_field_ident, &mut descendents, roles, selected_keys, values)?
                                        }
                                    )}
                   }
               );
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;

                // Add if columns should not be skipped
                if !join_attrs.skip_mut_self_cols {
                    self.insert_columns_code.push(quote!(
                        for other_column in <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns() {
                                #default_self_column_code;
                                let self_column = #columns_map_code;
                                e.push_literal(self_column);
                                e.push_literal(", ");
                        }
                    ));

                    self.insert_values_code.push(
                        match field.number_of_options  {
                                    2 => { // Option<Option<T>>
                                            quote!(
                                                if let Some(field) = &self. #rust_field_ident {
                                                    if let Some(f) = field {
                                                        toql :: key :: Key :: params(& < #rust_type_ident as toql ::
                                                                                        keyed :: Keyed > ::
                                                                                        key(f))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                                    } else {
                                                        <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                        .iter().for_each(|_| {
                                                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                                                values.push_literal(", ");});

                                                    }
                                                } else {
                                                    <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|_| { values.push_literal("DEFAULT, ");});
                                                }

                                            )
                                        },
                                    1 if field.preselect => { // #[toql(preselect)] Option<T> 
                                    // TODO Option wrapping
                                        quote!(
                                            if let Some(f) =  &self. #rust_field_ident {
                                                        toql :: key :: Key :: params(& < #rust_type_ident as toql ::
                                                                                        keyed :: Keyed > ::
                                                                                        key(f))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                                    } else {
                                                        <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                        .iter().for_each(|_| {
                                                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                                                values.push_literal(", ");
                                                                });
                                                    }
                                            )
                                    },

                                    1 if !field.preselect => { // Option<T> selectable 
                                        quote!(
                                            if let Some(field) = &self. #rust_field_ident {
                                                        toql :: key :: Key :: params(& < #rust_type_ident as toql ::
                                                                                        keyed :: Keyed > ::
                                                                                        key(field))
                                                                                        .iter()
                                                                                        .for_each(|p| {
                                                                                            values.push_arg(p.to_owned());
                                                                                            values.push_literal(", ");
                                                                                            });
                                            } else {
                                                <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                                    .iter().for_each(|_|  {values.push_literal("DEFAULT, ");});
                                            }
                                        )
                                    },
                                    _ => { // T
                                        quote!(
                                            &toql::key::Key::params( &<#rust_type_ident as toql::keyed::Keyed>::key(&self. #rust_field_ident))
                                        .into_iter() .for_each(|a| {values.push_arg(a); values.push_literal(", " );});
                                    )
                                    }
                                }
                    );
                }
                if join_attrs.key && !field.skip_mut {
                    self.duplicate = true;
                }
            }
            FieldKind::Merge(_merge) => {
                if field.skip_mut {
                    return Ok(());
                }
                let rust_base_type_ident = &field.rust_base_type_ident;
                // TODO throw error if we dispatch dispatch beyond first merge
                self.dispatch_columns_code.push(
                   quote!(
                        #toql_field_name => {
                             return Ok(<#rust_base_type_ident as toql::tree::tree_insert::TreeInsert>::columns(&mut descendents)?);
                        }
                )
               );
                self.dispatch_values_code.push(
                    match field.number_of_options {
                        0 => {
                            quote!(
                                #toql_field_name => {
                                    for f in &self. #rust_field_ident{
                                        <#rust_base_type_ident as toql::tree::tree_insert::TreeInsert>::values(f, &mut descendents, roles, selected_keys, values)?
                                    }
                                }
                            )
                        }
                        _ => {quote!( // must be 1
                                #toql_field_name => {
                                    if let Some (fs) = self. #rust_field_ident .as_ref(){
                                        for f in fs {
                                            <#rust_base_type_ident as toql::tree::tree_insert::TreeInsert>::values(f, &mut descendents, roles, selected_keys, values)?
                                        }
                                    }
                                }
                        )},
                    }
               );
            }
        };

        Ok(())
    }
}
impl<'a> quote::ToTokens for CodegenInsert<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;
        let dispatch_columns_code = &self.dispatch_columns_code;
        let dispatch_values_code = &self.dispatch_values_code;
        let insert_columns_code = &self.insert_columns_code;
        let insert_values_code = &self.insert_values_code;

        let role_assert = match self.struct_ins_roles {
            None => quote!(),
            Some(role_expr_string) => {
                quote!(
                    if !toql::role_validator::RoleValidator::is_valid(roles, &&toql::role_expr_macro::role_expr!(#role_expr_string))  {
                        return Err( toql::sql_builder::sql_builder_error::SqlBuilderError::RoleRequired(#role_expr_string .to_string()).into())
                    }
                )
            }
        };

        let mods = quote! {
                impl toql::tree::tree_insert::TreeInsert for #struct_ident {

                    #[allow(unused_mut)]
                    fn columns<'a, I>(  mut descendents: &mut I)
                            -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                            {

                        let mut e = toql::sql_expr::SqlExpr::new();
                         match descendents.next() {
                               Some(d) => match d.as_str() {
                                   #(#dispatch_columns_code),*
                                   f @ _ => {
                                        return Err(
                                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                    }
                               },
                               None => {
                                   e.push_literal("(");
                                   #(#insert_columns_code)*
                                   e.pop_literals(2);
                                   e.push_literal(")");
                               }
                        }
                        Ok(e)
                    }
                    #[allow(unused_mut, unused_variables)]
                    fn values<'a, I>(&self,
                                        mut descendents: &mut I,
                                        roles: &std::collections::HashSet<String>,
                                        selected_keys: Option<&[Vec<toql::sql_arg::SqlArg>]>,
                                         values:  &mut toql::sql_expr::SqlExpr
                                ) -> std::result::Result<(),  toql::error::ToqlError>
                                 where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                                {

                                    match descendents.next() {
                                        Some(d) => match d.as_str() {
                                            #(#dispatch_values_code),*
                                            f @ _ => {
                                                    return Err(
                                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                                }
                                        },
                                        None => {
                                            #role_assert

                                             if let Some(kl) = selected_keys {
                                                if !kl.contains(&toql::key::Key::params(&toql::keyed::Keyed::key(self))) {
                                                    return Ok(());
                                                }
                                            } 

                                            values.push_literal("(");
                                            #(#insert_values_code)*
                                            values.pop_literals(2);
                                            values.push_literal("), ");
                                        }
                                    }
                                    Ok(())
                                }
                }

                  impl toql::tree::tree_insert::TreeInsert for &#struct_ident {

                    #[allow(unused_mut)]
                    fn columns<'a, I>(  mut descendents: &mut I)
                            -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> {
                                <#struct_ident as toql::tree::tree_insert::TreeInsert>::columns(descendents)
                            }
                    #[allow(unused_mut)]
                     fn values<'a, I>(&self,
                                        mut descendents: &mut  I,
                                        roles: &std::collections::HashSet<String>,
                                        selected_keys: Option<&[Vec<toql::sql_arg::SqlArg>]>,
                                         values:  &mut toql::sql_expr::SqlExpr
                                ) -> std::result::Result<(),  toql::error::ToqlError>
                                 where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                                {
                                    <#struct_ident as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, selected_keys, values)
                                }
                  }
                  impl toql::tree::tree_insert::TreeInsert for &mut #struct_ident {

                    #[allow(unused_mut)]
                    fn columns<'a, I>(  mut descendents: &mut I)
                            -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
                             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> {
                                <#struct_ident as toql::tree::tree_insert::TreeInsert>::columns(descendents)
                            }
                    #[allow(unused_mut)]
                     fn values<'a, I>(&self,
                                        mut descendents: &mut  I,
                                        roles: &std::collections::HashSet<String>,
                                        selected_keys: Option<&[Vec<toql::sql_arg::SqlArg>]>,
                                         values:  &mut toql::sql_expr::SqlExpr
                                ) -> std::result::Result<(),  toql::error::ToqlError>
                                 where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                                {
                                    <#struct_ident as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, selected_keys, values)
                                }
                  }
        };

        tracing::debug!(
            "Source code for `{}`:\n{}",
            self.struct_ident,
            mods.to_string()
        );
        tokens.extend(mods);
    }
}
