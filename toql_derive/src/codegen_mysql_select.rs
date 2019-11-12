use proc_macro2::{Span, TokenStream};
use syn::Ident;

use heck::MixedCase;

use darling::{Error, Result};

use crate::sane::{FieldKind, SqlTarget};

pub(crate) struct GeneratedMysqlSelect<'a> {
    struct_ident: &'a Ident,
    sql_table_name: String,
    sql_table_alias: String,

    select_columns: Vec<String>,
    select_columns_params: Vec<TokenStream>,

    select_joins: Vec<String>,
    select_joins_params: Vec<TokenStream>,

    select_keys: Vec<TokenStream>,

    select_key_types: Vec<TokenStream>,
    select_keys_params: Vec<TokenStream>,

    merge_code: Vec<TokenStream>,

    merge_fields: Vec<crate::sane::Field>,
    merge_self_fields: Vec<String>,
}

impl<'a> GeneratedMysqlSelect<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> GeneratedMysqlSelect {
        GeneratedMysqlSelect {
            struct_ident: &toql.rust_struct_ident,
            sql_table_name: toql.sql_table_name.to_owned(),
            sql_table_alias: toql.sql_table_alias.to_owned(),

            select_columns: Vec::new(),
            select_columns_params: Vec::new(),

            select_joins: Vec::new(),
            select_joins_params: Vec::new(),

            select_keys: Vec::new(),

            select_key_types: Vec::new(),
            select_keys_params: Vec::new(),

            merge_code: Vec::new(),
            merge_fields: Vec::new(),
            merge_self_fields: Vec::new(),
        }
    }

    pub fn add_select_field(&mut self, field: &crate::sane::Field) -> Result<()> {
        let rust_type_ident = &field.rust_type_ident;
        let rust_field_name = &field.rust_field_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if regular_attrs.key {
                    let key_index = syn::Index::from(self.select_keys.len());
                    if let SqlTarget::Column(ref sql_column) = &regular_attrs.sql_target {
                        let key_expr = format!("{}.{} = ?", self.sql_table_alias, sql_column);
                        self.select_keys.push(quote!(#key_expr));
                    } else {
                        return Err(Error::custom(
                            "SQL expression not allowed for key. Remove `sql` from #[toql(..)]",
                        ));
                    }

                    self.select_keys_params.push(quote! {
                        params.push( key . #key_index .to_string());
                    });

                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                match &regular_attrs.sql_target {
                    SqlTarget::Expression(ref _expression) => {
                        // TODO
                    }
                    SqlTarget::Column(ref sql_column) => {
                        self.select_columns
                            .push(format!("{}.{}", &self.sql_table_alias, sql_column));
                    }
                };
            }
            FieldKind::Join(ref join_attrs) => {
                let columns_map_code = &join_attrs.columns_map_code;
                let default_self_column_code = &join_attrs.default_self_column_code;

                self.select_columns.push(String::from("true"));
                self.select_columns.push(String::from("{}"));
                self.select_columns_params
                    .push(quote!(#rust_type_ident :: columns_sql()));

                if join_attrs.key {
                    let key_index = syn::Index::from(self.select_keys.len());
                    let aliased_column_format = format!("{}.{{}} = ?", &self.sql_table_alias);
                    self.select_keys.push(quote!( {
                        &<#rust_type_ident as toql::key::Key>::columns().iter()
                        .map(|other_column|{
                            #default_self_column_code;
                            let self_column = #columns_map_code;
                            format!(#aliased_column_format, self_column )
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                    ));
                    self.select_keys_params.push(  quote! {
                            params.extend_from_slice( &<#rust_type_ident as toql::key::Key>::params( &key. #key_index));
                        });
                    self.merge_self_fields.push(rust_field_name.to_string());
                }

                self.select_joins.push(format!(
                    "JOIN {} {} ON ({{}}{{}}) {{}}",
                    join_attrs.sql_join_table_name, rust_field_name
                ));

                let select_join_params_format = format!(
                    "{}.{{}} = {}.{{}}",
                    &self.sql_table_alias, join_attrs.join_alias
                );

                self.select_joins_params.push(quote!(
                    {


                      <#rust_type_ident as toql::key::Key>::columns().iter()

                        .map(|other_column| {
                            #default_self_column_code;
                            let self_column= #columns_map_code;

                        format!(#select_join_params_format, self_column, other_column)
                        }).collect::<Vec<String>>().join(" AND ")
                    }
                ));
                self.select_joins_params
                    .push(if let Some(ref sql) = &join_attrs.on_sql {
                        let on_sql = format!(
                            " AND ({})",
                            sql.replace("..", &format!("{}.", join_attrs.join_alias))
                        );
                        quote!( #on_sql)
                    } else {
                        quote!("")
                    });

                self.select_joins_params
                    .push(quote!(#rust_type_ident :: joins_sql()));
            }
            FieldKind::Merge(_) => {
                self.merge_fields.push(field.clone());
            }
        };
        Ok(())
    }

    pub fn build_merge(&mut self) {
        // Build all merge fields
        // This must be done after the first pass, becuase all key names must be known at this point
        for field in &self.merge_fields {
            match &field.kind {
                FieldKind::Merge(merge_attrs) => {
                    let mut on_condition: Vec<String> = Vec::new();
                    if let Some(sql) = &merge_attrs.on_sql {
                        // resolve ..
                        on_condition.push(
                            sql.replace("..", &format!("{}.", merge_attrs.join_alias).to_owned()),
                        );
                    }

                    // Build join for all keys of that struct
                    for self_field in &self.merge_self_fields {
                        let default_other_field =
                            format!("{}_{}", field.rust_type_name.to_mixed_case(), &self_field);
                        let other_field = merge_attrs.other_field(&self_field, default_other_field);

                        let self_column = merge_attrs.column(&self_field);
                        let other_column = merge_attrs.column(&other_field);

                        on_condition.push(format!(
                            "{}.{} = {}.{}",
                            merge_attrs.join_alias,
                            self_column,
                            merge_attrs.join_alias,
                            other_column
                        ));
                    }

                    let merge_join = format!(
                        "JOIN {} {} ON ({} AND {{}})",
                        merge_attrs.sql_join_table_name,
                        merge_attrs.join_alias,
                        on_condition.join(" AND ")
                    );

                    let struct_ident = self.struct_ident;
                    let merge_function = Ident::new(
                        &format!("merge_{}", &field.rust_field_name),
                        Span::call_site(),
                    );

                    let merge_field_init = if field.number_of_options > 0 {
                        quote!(Some(Vec::new()))
                    } else {
                        quote!(Vec::new())
                    };
                    let rust_field_ident = &field.rust_field_ident;
                    let rust_type_ident = &field.rust_type_ident;

                    self.merge_code.push(quote!(
                                let #rust_field_ident = #rust_type_ident :: select_dependencies( &format!(#merge_join, key_predicate), &params, conn)?;
                                for e in entities.iter_mut() { e . #rust_field_ident = #merge_field_init; }
                                #struct_ident :: #merge_function(&mut entities, #rust_field_ident);
                        ));
                }
                _ => {
                    panic!("Should be never called!");
                }
            }
        }
    }
}

impl<'a> quote::ToTokens for GeneratedMysqlSelect<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = self.struct_ident;

        let sql_table_name = &self.sql_table_name;
        let sql_table_alias = &self.sql_table_alias;

        let select_columns = self.select_columns.join(",");

        let select_columns_params = &self.select_columns_params;
        let select_joins = self.select_joins.join(" ");

        let select_joins_params = &self.select_joins_params;

        let select_statement = format!(
            "SELECT {{}} FROM {} {} {{}}{{}}",
            sql_table_name, sql_table_alias
        );

        let merge_code = &self.merge_code;

        let select_keys = &self.select_keys;
        let select_keys_params = &self.select_keys_params;

        let columns_sql_code = if select_columns_params.is_empty() {
            quote!( String::from(#select_columns))
        } else {
            quote!(format!(#select_columns, #(#select_columns_params),*))
        };
        let joins_sql_code = if select_joins_params.is_empty() {
            quote!( String::from(#select_joins))
        } else {
            quote!(format!(#select_joins, #(#select_joins_params),*))
        };

        let select = quote! {
                impl<'a> toql::mysql::select::Select<#struct_ident> for #struct_ident {


                    fn columns_sql() -> String {
                           #columns_sql_code

                    }
                    fn joins_sql() -> String {
                            #joins_sql_code
                    }
                    fn select_sql(join: Option<&str>) -> String {
                            format!( #select_statement,
                            Self::columns_sql(), Self::joins_sql(),join.unwrap_or(""))
                    }


                     fn select_one<C>(key: &<#struct_ident as toql::key::Key>::Key, conn: &mut C)
                     -> Result<#struct_ident,  toql::error::ToqlError>
                     where C: toql::mysql::mysql::prelude::GenericConnection
                     {
                        let select_stmt = format!( "{} WHERE {} LIMIT 0,2", Self::select_sql(None), vec![ #(#select_keys),*].join( " AND "));

                        let mut params :Vec<String> = Vec::new();

                        #(#select_keys_params)*
                        toql::log_sql!(select_stmt, params);

                        let entities_stmt = conn.prep_exec(select_stmt, &params)?;
                        let mut entities = toql::mysql::row::from_query_result::< #struct_ident>(entities_stmt)?;

                        if entities.len() > 1 {
                            return Err( toql::error::ToqlError::NotUnique);
                        } else if entities.is_empty() {
                            return Err( toql::error::ToqlError::NotFound);
                        }

                        let key_predicate = vec![ #(#select_keys),*].join( " AND ");
                        #(#merge_code)*
                        Ok(entities.pop().unwrap())
                     }


                        fn select_many<C>(
                            key: &<#struct_ident as toql::key::Key>::Key,
                            conn: &mut C,
                            first: u64,
                            max: u16
                        ) -> Result<Vec< #struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                        {
                                unimplemented!();


                        }

                        fn select_dependencies<C>(join: &str, params:&Vec<String>,
                            conn: &mut C) -> Result<Vec<#struct_ident> ,  toql::error::ToqlError>
                            where C: toql::mysql::mysql::prelude::GenericConnection
                            {
                                let select_stmt =  Self::select_sql(Some(join));

                        toql::log_sql!(select_stmt, params);

                        let entities_stmt = conn.prep_exec(select_stmt, params)?;
                        let mut entities = toql::mysql::row::from_query_result::<#struct_ident>(entities_stmt)?;


                        let key_predicate = vec![ #(#select_keys),*].join( " AND ");
                        #(#merge_code)*

                        Ok(entities)
                        }

                }

        };

        log::debug!(
            "Source code for `{}`:\n{}",
            self.struct_ident,
            select.to_string()
        );
        tokens.extend(select);
    }
}
