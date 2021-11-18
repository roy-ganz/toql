use quote::quote;

use crate::parsed::{
    field::{
        field_kind::FieldKind,
        join_field::JoinSelection,
        merge_field::{MergeColumn, MergeSelection},
        regular_field::{RegularSelection, SqlTarget},
    },
    parsed_struct::ParsedStruct,
};

use heck::SnakeCase;
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut key_field_names = Vec::new();
    let mut field_mappings = Vec::new();
    let struct_name_ident = &parsed_struct.struct_name;
    let struct_name = &parsed_struct.struct_name.to_string();

    for (name, arg) in &parsed_struct.selections {
        let fields = &arg.fields;

        field_mappings.push(quote!(
                mapper.map_selection( #name, toql::fields_macro::fields!(#struct_name_ident, #fields).into_inner());
            ));
    }

    for (name, arg) in &parsed_struct.predicates {
        let toql_field_name = &name;
        let sql_mapping = &arg.sql;

        let on_aux_params: Vec<TokenStream> = arg
            .on_aux_params
            .iter()
            .map(|(name, index)| quote!(.on_aux_param( #index, String::from(#name))))
            .collect::<Vec<_>>();

        let countfilter_ident = if arg.count_filter {
            quote!( .count_filter(true))
        } else {
            quote!()
        };
        let handler = if let Some(handler) = &arg.handler {
            quote!(.handler(#handler ()))
        } else {
            quote!()
        };

        field_mappings.push(quote! {
                        mapper.map_predicate_with_options(
                        #toql_field_name,
                        toql::sql_expr_macro::sql_expr!(#sql_mapping),
                        toql::table_mapper::predicate_options::PredicateOptions::new() #(#on_aux_params)* #countfilter_ident #handler
                        );
                    });
    }

    let count_filter_code = quote!();

    for field in &parsed_struct.fields {
        let field_name_ident = &field.field_name;

        let load_restriction_code = match &field.roles.load {
            Some(role) => quote! {  .restrict_load(toql::role_expr_macro::role_expr!(#role)) },
            None => quote!(),
        };
        let update_restriction_code = match &field.roles.update {
            Some(role) => quote! {  .restrict_update(toql::role_expr_macro::role_expr!(#role)) },
            None => quote!(),
        };

        // Joined field
        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Join(join_kind) => {
                let columns_map_code = &join_kind.columns_map_code;
                let default_self_column_code = &join_kind.default_self_column_code;
                let field_base_type = &field.field_base_type;
                let toql_field_name = &field.toql_query_name;
                let sql_join_mapper_name = &field.field_base_name.to_string();

                let sql_join_table_name = &join_kind.sql_join_table_name;

                // Build predicate based on key information or custom provided column pairs
                let col_array = if join_kind.columns.is_empty() {
                    quote!(<<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns())
                } else {
                    let other_columns: Vec<String> = join_kind
                        .columns
                        .iter()
                        .map(|column| String::from(column.other.as_str()))
                        .collect::<Vec<_>>();
                    quote!( [ #(String::from(#other_columns)),* ])
                };
                let on_predicate = if let Some(on) = &join_kind.on_sql {
                    quote!(t.extend(toql::sql_expr_macro::sql_expr!(#on)))
                } else {
                    quote!(t.pop_literals(5)) // Remove unneeded ' AND '
                };
                let join_predicate = quote!(
                 #col_array .iter()
                .for_each(|other_column| {
                    #default_self_column_code;
                    let self_column= #columns_map_code;
                    t.push_self_alias();
                    t.push_literal(".");
                    t.push_literal(self_column);
                    t.push_literal(" = ");
                    t.push_other_alias();
                    t.push_literal(".");
                    t.push_literal(other_column);
                    t.push_literal(" AND ");
                });
                 #on_predicate

                 );

                let join_type = if join_kind.selection == JoinSelection::SelectInner
                    || join_kind.selection == JoinSelection::PreselectInner
                {
                    quote!(toql::table_mapper::join_type::JoinType::Inner)
                } else {
                    quote!(toql::table_mapper::join_type::JoinType::Left)
                };

                let preselect_ident = if join_kind.selection == JoinSelection::PreselectLeft
                    || join_kind.selection == JoinSelection::PreselectInner
                {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };

                let partial_table_ident = if join_kind.partial_table {
                    quote!( .partial_table(true))
                } else {
                    quote!()
                };

                let skip_mut_ident = if field.skip_mut {
                    quote!( .skip_mut(true))
                } else {
                    quote!()
                };

                let key_ident = if join_kind.key {
                    quote!( .key(true))
                } else {
                    quote!()
                };

                let aux_params = join_kind
                    .aux_params
                    .iter()
                    .map(|(name,value)| {
                        quote!(.aux_param(String::from(#name), String::from(#value)))
                    })
                    .collect::<Vec<TokenStream>>();

                let handler = if let Some(handler) = join_kind.handler.as_ref() {
                    quote!( .handler(#handler ()))
                } else {
                    quote!()
                };

                let sql_join_table_name_ident =
                    syn::Ident::new(&sql_join_table_name, proc_macro2::Span::call_site());

                field_mappings.push(quote! {
                    mapper.map_join_with_options(#toql_field_name, #sql_join_mapper_name,
                    #join_type,
                    {let mut t = toql::sql_expr::SqlExpr::literal(< #sql_join_table_name_ident as toql::table_mapper::mapped::Mapped>::table_name()); t.push_literal(" "); t.push_other_alias(); t }, 
                    { let mut t = toql::sql_expr::SqlExpr::new(); #join_predicate; t },
                     toql::table_mapper::join_options::JoinOptions::new() #(#aux_params)*
                     #preselect_ident #key_ident #skip_mut_ident #load_restriction_code #partial_table_ident #handler
                    );
                });

                if join_kind.key {
                    key_field_names.push(field_name_ident.to_string());
                }
            }
            FieldKind::Regular(regular_kind) => {
                let toql_field_name = &field.toql_query_name;

                let preselect_ident = if regular_kind.selection
                    == RegularSelection::PreselectNullable
                    || regular_kind.selection == RegularSelection::Preselect
                {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };
                let skip_wc_ident = if regular_kind.skip_wildcard {
                    quote!( .skip_wildcard(true))
                } else {
                    quote!()
                };

                let skip_mut_ident = if field.skip_mut {
                    quote!(.skip_mut(true))
                } else {
                    quote!()
                };
                let key_ident = if regular_kind.key {
                    quote!(.key(true))
                } else {
                    quote!()
                };

                let aux_params = regular_kind
                    .aux_params
                    .iter()
                    .map(|(name, value)| {
                        quote!(.aux_param(String::from(#name), String::from(#value)))
                    })
                    .collect::<Vec<_>>();

                let handler = if let Some(handler) = regular_kind.handler.as_ref() {
                    quote!( .handler(#handler ()))
                } else {
                    quote!()
                };

                field_mappings.push( match &regular_kind.sql_target {
                    SqlTarget::Expression(ref expression) => {
                        quote! {
                            mapper.map_expr_with_options( #toql_field_name,  toql::sql_expr_macro::sql_expr!( #expression),
                        toql::table_mapper::field_options::FieldOptions::new() #(#aux_params)*
                            #preselect_ident #skip_wc_ident #load_restriction_code #update_restriction_code #skip_mut_ident #handler);
                        }
                    }
                    SqlTarget::Column(ref column) => {
                        quote! {
                            mapper.map_column_with_options( #toql_field_name, #column,
                            toql::table_mapper::field_options::FieldOptions::new() #(#aux_params)*
                            #key_ident #preselect_ident #skip_wc_ident #load_restriction_code #update_restriction_code #skip_mut_ident #handler);
                            }
                    }
                });

                if regular_kind.key {
                    key_field_names.push(field_name_ident.to_string());
                }
            }
            FieldKind::Merge(ref merge_kind) => {
                let toql_field_name = &field.toql_query_name;
                let sql_merge_mapper_name = &field.field_base_name.to_string();

                let table_name = &parsed_struct.table;
                let table_join = quote!(toql::sql_expr::SqlExpr::from(vec![
                     toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                     toql::sql_expr::SqlExprToken::Literal(#table_name.to_string()),
                     toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                     toql::sql_expr::SqlExprToken::SelfAlias
                ]));

                let join_statement = if let Some(custom_join) = &merge_kind.join_sql {
                    quote!(
                      { let mut t = toql::sql_expr_macro::sql_expr!(#custom_join);
                      t.extend(toql::sql_expr::SqlExpr::literal(" "))
                      .extend(#table_join); t}
                    )
                } else {
                    table_join
                };

                // Build join predicate
                // - use custom predicate if provided
                // - build from columns, if provided
                // - build from key, if columns are missing

                let join_predicate = if let Some(custom_on) = &merge_kind.on_sql {
                    quote!(toql::sql_expr_macro::sql_expr!( #custom_on))
                } else if merge_kind.columns.is_empty() {
                    let self_key_ident = syn::Ident::new(
                        &format!("{}Key", &parsed_struct.struct_name),
                        proc_macro2::Span::call_site(),
                    );

                    quote!(  {
                    let mut tokens: Vec<toql::sql_expr::SqlExprToken>= Vec::new();
                        <#self_key_ident as toql::key::Key>::columns().iter()
                        .zip(<#self_key_ident as toql::key::Key>::default_inverse_columns()).for_each(|(t,o)| {
                        tokens.extend(vec![toql::sql_expr::SqlExprToken::SelfAlias,
                        toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                        toql::sql_expr::SqlExprToken::Literal(t.to_string()),
                        toql::sql_expr::SqlExprToken::Literal(" = ".to_string()),
                        toql::sql_expr::SqlExprToken::OtherAlias,
                        toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                        toql::sql_expr::SqlExprToken::Literal(o.to_string()),
                        toql::sql_expr::SqlExprToken::Literal( " AND ".to_string())
                        ].into_iter())});
                        tokens.pop(); // ' AND '
                        toql::sql_expr::SqlExpr::from(tokens)
                    })
                } else {
                    let mut default_join_predicate: Vec<TokenStream> = Vec::new();
                    default_join_predicate
                        .push(quote!(  let mut t =  toql::sql_expr::SqlExpr::new();));

                    let composite = merge_kind.columns.len() > 1;
                    for m in &merge_kind.columns {
                        let this_column = &m.this;
                        default_join_predicate.push(quote!(
                                        t.push_self_alias();
                                        t.push_literal(".");
                                        t.push_literal(#this_column);
                                        t.push_literal(" = "); ));
                        match &m.other {
                            MergeColumn::Aliased(a) => {
                                default_join_predicate.push(quote!( t.push_literal(#a);));
                            }
                            MergeColumn::Unaliased(u) => default_join_predicate.push(quote!(
                             t.push_other_alias();
                             t.push_literal(".");
                             t.push_literal(#u);
                            )),
                        }
                        if composite {
                            default_join_predicate.push(quote!(t.push_literal(" AND ");));
                        }
                    }

                    if composite {
                        // Remove last ' AND '
                        default_join_predicate.push(quote!(t.pop_literals(5);));
                    }
                    default_join_predicate.push(quote!(t));

                    quote!(#(#default_join_predicate)*)
                };

                let preselect_ident = if merge_kind.selection == MergeSelection::Preselect {
                    quote!( .preselect(true))
                } else {
                    quote!()
                };

                field_mappings.push(quote! {
                        mapper.map_merge_with_options(#toql_field_name, #sql_merge_mapper_name,
                           {#join_statement},
                            { #join_predicate },
                            toql::table_mapper::merge_options::MergeOptions::new() #preselect_ident #load_restriction_code
                            );
                });
            }
        };
    }
    // Generate token stream

    let delete_role_code = match &parsed_struct.roles.delete {
        Some(r) => quote!(mapper.restrict_delete( toql::role_expr_macro::role_expr!(#r)); ),
        None => quote!(),
    };
    let load_role_code = match &parsed_struct.roles.load {
        Some(r) => quote!(mapper.restrict_load( toql::role_expr_macro::role_expr!(#r)); ),
        None => quote!(),
    };

    let sql_table_name = &parsed_struct.table;
    let sql_table_alias = parsed_struct.table.to_snake_case();

    let builder = quote!(

        impl toql::table_mapper::mapped::Mapped for #struct_name_ident {

            fn type_name() -> String {
                String::from(#struct_name)
            }

            fn table_name() -> String {
                String::from(#sql_table_name)
            }
            fn table_alias() -> String {
                String::from(#sql_table_alias)
            }
            #[allow(redundant_semicolons)]
            fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()>{

                #(#field_mappings)*

                #count_filter_code

                #load_role_code
                #delete_role_code
                Ok(())
            }
        }

         impl toql::table_mapper::mapped::Mapped for &#struct_name_ident {

              fn type_name() -> String {
              <#struct_name_ident as toql::table_mapper::mapped::Mapped>::type_name()
            }

            fn table_name() -> String {
               <#struct_name_ident as toql::table_mapper::mapped::Mapped>::table_name()
            }
            fn table_alias() -> String {
                <#struct_name_ident as toql::table_mapper::mapped::Mapped>::table_alias()
            }
            fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()>{
                <#struct_name_ident as toql::table_mapper::mapped::Mapped>::map(mapper)
            }
         }

         impl toql::table_mapper::mapped::Mapped for &mut #struct_name_ident {

              fn type_name() -> String {
              <#struct_name_ident as toql::table_mapper::mapped::Mapped>::type_name()
            }

            fn table_name() -> String {
               <#struct_name_ident as toql::table_mapper::mapped::Mapped>::table_name()
            }
            fn table_alias() -> String {
                <#struct_name_ident as toql::table_mapper::mapped::Mapped>::table_alias()
            }
            fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()>{
                <#struct_name_ident as toql::table_mapper::mapped::Mapped>::map(mapper)
            }
         }

    );

    log::debug!(
        "Source code for `{}`:\n{}",
        &struct_name,
        builder.to_string()
    );

    tokens.extend(builder);
}
