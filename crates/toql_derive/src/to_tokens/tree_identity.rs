use crate::parsed::{
    field::{
        field_kind::FieldKind,
        join_field::JoinSelection,
        merge_field::{MergeColumn, MergeSelection},
        regular_field::SqlTarget,
    },
    parsed_struct::ParsedStruct,
};
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut dispatch_auto_id_code = Vec::new();
    let mut dispatch_identity_code = Vec::new();
    let mut identity_set_merges_key_code = Vec::new();
    let mut key_columns = Vec::new();
    let mut number_of_keys = 0;

    let struct_name_ident = &parsed_struct.struct_name;
    let struct_name = &parsed_struct.struct_name.to_string();

    for field in &parsed_struct.fields {
        let field_ident = &field.field_name;
        let field_base_name = &field.field_base_name.to_string();
        let toql_query_name = &field.toql_query_name;
        let field_base_type = &field.field_base_type;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(field_attrs) => {
                if field_attrs.key {
                    number_of_keys += 1;
                    if let SqlTarget::Column(column) = &field_attrs.sql_target {
                        key_columns.push(column.to_string())
                    }
                }
            }
            FieldKind::Join(join_attrs) => {
                if join_attrs.key {
                    number_of_keys += 1;
                }

                dispatch_identity_code.push(
                    match join_attrs.selection {
                         JoinSelection::SelectLeft => {
                            quote!(
                                #toql_query_name => {
                                    if let Some(e) = self. #field_ident .as_mut() {
                                        if let Some(e2) = e .as_mut() {
                                            toql::tree::tree_identity::TreeIdentity::set_id(e2, descendents, action)?;
                                        }
                                    }
                                }
                            )
                         }

                        JoinSelection::PreselectInner => {
                            quote!(
                                #toql_query_name => {
                                    toql::tree::tree_identity::TreeIdentity::set_id(&mut self. #field_ident, descendents, action)?;
                                }
                            )
                        }
                        JoinSelection::PreselectLeft | JoinSelection::SelectInner => {
                            quote!(
                                #toql_query_name => {
                                    if let Some(e) = self. #field_ident. as_mut() {
                                        toql::tree::tree_identity::TreeIdentity::set_id(e, descendents, action)?;
                                    }
                                }
                            )
                        }
                    }
                );
                dispatch_auto_id_code.push(
                   quote!(
                       #toql_query_name => {
                            Ok(<#field_base_type as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)?)
                        }
                )
               );

                if join_attrs.partial_table {
                    let inverse_columns_mapping = join_attrs
                        .columns
                        .iter()
                        .map(|column| {
                            let tc = &column.this;
                            let oc = &column.other;
                            quote!(#oc => String::from(#tc), )
                        })
                        .collect::<Vec<_>>();

                    // Assume same key columns for partial tables and
                    // default invese columns for regular joins
                    let default_inverse_columns = if join_attrs.partial_table {
                        quote!(&self_key_columns)
                    } else {
                        quote!(<< #field_base_type as toql :: keyed ::   Keyed > :: Key as toql :: key :: Key > ::
                        default_inverse_columns() )
                    };
                    identity_set_merges_key_code.push(
                        quote!(
                             if let Some(mut f) = self. #field_ident .as_mut() {
                        // Get inverse columns
                        let default_inverse_columns = #default_inverse_columns;

                        let inverse_columns =
                            <<#field_base_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    let inverse_column = match c.as_str() {
                                        #(#inverse_columns_mapping)*
                                        _ => default_inverse_columns.get(i).unwrap().to_owned(),
                                    };
                                    inverse_column
                                })
                                .collect::<Vec<String>>();
                       // Build target key args
                        let mut args = Vec::new();
                        for c in inverse_columns {
                            let i = self_key_columns.iter().position(|r| r == &c)
                                .ok_or_else(|| toql::table_mapper::error::TableMapperError::ColumnMissing(#field_base_name.to_string(), c.to_string()))?;
                            args.push(self_key_params.get(i).ok_or_else(||toql::table_mapper::error::TableMapperError::ColumnMissing(#field_base_name.to_string(), c.to_string()))?.to_owned());
                        }

                        let action = toql::tree::tree_identity::IdentityAction::Set(std::cell::RefCell::new(args));
                        toql::tree::tree_identity::TreeIdentity::set_id(f, std::iter::empty(), &action)?;
                             }
                        )
                    );
                }
            }
            FieldKind::Merge(merge) => {
                dispatch_auto_id_code.push(quote!(
                       #toql_query_name => {
                                Ok(<#field_base_type as toql::tree::tree_identity::TreeIdentity>::
                                auto_id(descendents)?)
                        }
                ));
                match merge.selection {
                    MergeSelection::Select => {
                        dispatch_identity_code.push(quote!(
                       #toql_query_name => {
                           if let Some(fs) = self. #field_ident. as_mut() {
                            for f in fs {
                                toql::tree::tree_identity::TreeIdentity::set_id(f, descendents.clone(),  action.clone())?
                            }
                           }
                        }
                ));
                    }
                    MergeSelection::Preselect => {
                        // Vec<T>
                        dispatch_identity_code.push(quote!(
                       #toql_query_name => {
                            for f in &mut self. #field_ident  {
                                toql::tree::tree_identity::TreeIdentity::set_id(f, descendents.clone(),  action.clone())?
                            }
                        }
                ));
                    }
                }

                let mut columns_merge = Vec::new();
                for c in &merge.columns {
                    //let this_col = c.this;
                    match &c.other {
                        MergeColumn::Aliased(a) => {
                            columns_merge.push(quote!(
                            key_expr.push_literal(#a);
                            ));
                        }
                        MergeColumn::Unaliased(u) => {
                            columns_merge.push(quote!(
                            key_expr.push_self_alias();
                               key_expr.push_literal(".");
                                  key_expr.push_literal(#u);

                            ));
                        }
                    }
                    columns_merge.push(quote!(
                    key_expr.push_literal(", ");
                    ));
                }

                let mut columns_code: Vec<TokenStream> = Vec::new();
                for c in &merge.columns {
                    columns_code.push(match &c.other {
                            MergeColumn::Aliased(a) => { quote!( columns.push(  toql :: sql_expr :: PredicateColumn::Literal(#a .to_owned())); )}
                            MergeColumn::Unaliased(a) => {quote!( columns.push(  toql :: sql_expr :: PredicateColumn::SelfAliased(#a .to_owned())); )}
                        });
                }

                let mut skip_identity_code = false;
                let mut column_mapping = Vec::new();
                if merge.columns.is_empty() {
                    for this_column in &key_columns {
                        let other_column = format!(
                            "{}_{}",
                            &heck::SnakeCase::to_snake_case(struct_name.to_string().as_str()),
                            this_column
                        );
                        column_mapping.push(quote!(
                            #this_column => #other_column

                        ));
                    }
                } else {
                    for c in &merge.columns {
                        let this_column = &c.this;
                        match &c.other {
                            MergeColumn::Aliased(_) => skip_identity_code = true,
                            MergeColumn::Unaliased(name) => {
                                column_mapping.push(quote!(
                                     #this_column => #name

                                ));
                            }
                        }
                    }
                }
                if !skip_identity_code {
                    match merge.selection {
                        MergeSelection::Preselect => {
                            identity_set_merges_key_code.push(
                                quote!(
                                    for e in &mut self. #field_ident {
                                        let key = toql :: keyed :: Keyed ::key(e);
                                        let merge_key = toql::keyed::Keyed::key(e);
                                        let merge_key_params = toql::key::Key::params(&merge_key);
                                        let valid = toql::sql_arg::valid_key(&merge_key_params);
                                        if matches!(
                                            action,
                                            toql::tree::tree_identity::IdentityAction::RefreshInvalid
                                        ) && valid
                                        {
                                        continue
                                        }
                                        if matches!(
                                            action,
                                            toql::tree::tree_identity::IdentityAction::RefreshValid
                                        ) && !valid
                                        {
                                        continue
                                        }
                                        let mut found = false;
                                        for (self_key_column, self_key_param) in self_key_columns.iter().zip(&self_key_params) {
                                            let calculated_other_column= match self_key_column.as_str() {
                                                #(#column_mapping,)*
                                                x @ _ => x
                                            };
                                            if cfg!(debug_assertions) {
                                                let foreign_identity_columns = <#field_base_type as toql::identity::Identity>::columns();
                                                if foreign_identity_columns.contains(&calculated_other_column.to_string()) {
                                                    found = true;
                                                }
                                            }
                                            toql::identity::Identity::set_column(e, calculated_other_column, self_key_param)?;
                                            if cfg!(debug_assertions) {
                                                if !found
                                                {
                                                    toql :: tracing :: warn !
                                                    ("`{}` is unable to update foreign key in `{}`. \
                                                            Try adding `#[toql(foreign_key)]` to field(s) in `{}` that refer to `{}`.",
                                                    #struct_name, #field_base_name,
                                                    #field_base_name, #struct_name)
                                                }
                                            }
                                        }
                                    }
                                )
                            )
                        }
                        MergeSelection::Select => {
                            identity_set_merges_key_code.push(
                                quote!(
                                    if let Some(u) = self. #field_ident .as_mut() {
                                        for e in u {
                                            let merge_key = toql::keyed::Keyed::key(e);
                                            let merge_key_params = toql::key::Key::params(&merge_key);
                                            let valid = toql::sql_arg::valid_key(&merge_key_params);
                                            if matches!(
                                                action,
                                                toql::tree::tree_identity::IdentityAction::RefreshInvalid
                                            ) && valid
                                            {
                                                continue
                                            }
                                            if matches!(
                                                action,
                                                toql::tree::tree_identity::IdentityAction::RefreshValid
                                            ) && !valid
                                            {
                                                continue
                                            }
                                            let mut found = false;
                                            for (self_key_column, self_key_param) in self_key_columns.iter().zip(&self_key_params) {
                                                let calculated_other_column= match self_key_column.as_str() {
                                                    #(#column_mapping,)*
                                                    x @ _ => x
                                                };
                                                 if cfg!(debug_assertions) {
                                                    let foreign_identity_columns = <#field_base_type as toql::identity::Identity>::columns();
                                                    if foreign_identity_columns.contains(&calculated_other_column.to_string()) {
                                                        found = true;
                                                    }
                                                }
                                                toql::identity::Identity::set_column(e, calculated_other_column, self_key_param)?;
                                                if cfg!(debug_assertions) {
                                                    if !found {
                                                        toql :: tracing :: warn !
                                                        ("`{}` is unable to update foreign key in `{}`. \
                                                                Try adding `#[toql(foreign_key)]` to field(s) in `{}` that refer to `{}`.",
                                                        #struct_name, #field_base_name,
                                                        #field_base_name, #struct_name)
                                                    }
                                                }
                                            }
                                        }

                                    }
                                )
                            )
                        }
                    }
                }
            }
        };
    }

    // Generate  token stream
    let identity_set_self_key_code = quote!(
       fn set_self_key(entity: &mut #struct_name_ident, args: &mut Vec<toql::sql_arg::SqlArg>, invalid_only: bool) -> std::result::Result<(), toql::error::ToqlError> {
                if invalid_only {
                     let self_key = toql::keyed::Keyed::key(&entity);
                     let self_key_params = toql::key::Key::params(&self_key);
                     if toql::sql_arg::valid_key(&self_key_params) {
                         return Ok(());
                     }
                }

                 let n = << #struct_name_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
                let end = args.len();
                let args: Vec<toql::sql_arg::SqlArg> =
                    args.drain(end - n..).collect::<Vec<_>>();

                 let key = std::convert::TryFrom::try_from(args)?;
                toql::keyed::KeyedMut::set_key(entity, key);
                 Ok(())
            }
            if let toql::tree::tree_identity::IdentityAction::SetInvalid(args) = action {
                set_self_key(self, &mut args.borrow_mut(), true)?;
            }
            if let toql::tree::tree_identity::IdentityAction::Set(args) = action {
               set_self_key(self, &mut args.borrow_mut(), false)?;
            }
    );

    let identity_set_key = quote!( #identity_set_self_key_code

                    let self_key = toql::keyed::Keyed::key(&self);
                    let self_key_params = toql::key::Key::params(&self_key);
                    let self_key_columns =  <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();


                   #(#identity_set_merges_key_code)*);

    let identity_auto_id_code = if number_of_keys == 1 && parsed_struct.auto_key {
        quote!(true)
    } else {
        quote!(false)
    };

    let mods = quote! {

            impl toql::tree::tree_identity::TreeIdentity for #struct_name_ident {
            #[allow(unused_mut)]
             fn auto_id< 'a, I >(mut descendents : I) -> std :: result :: Result < bool, toql::error::ToqlError >
             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> {
                  match descendents.next() {
                           Some(d) => match d.as_str() {
                               #(#dispatch_auto_id_code),*
                               f @ _ => {
                                    return Err(
                                        toql::error::ToqlError::SqlBuilderError (
                                         toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                        .into());
                                }
                           },
                           None => {
                              Ok(#identity_auto_id_code)
                           }
                    }

             }

             #[allow(unused_variables, unused_mut)]
             fn set_id < 'a, 'b, I >(&mut self, mut descendents : I, action: &'b toql::tree::tree_identity::IdentityAction)
                        -> std :: result :: Result < (), toql::error::ToqlError >
                        where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
            {
                     match descendents.next() {
                           Some(d) => match d.as_str() {
                               #(#dispatch_identity_code),*
                               f @ _ => {
                                    return Err(
                                        toql::error::ToqlError::SqlBuilderError (
                                         toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                        .into());
                                }
                           },
                           None => {
                             #identity_set_key
                           }
                    }
                    Ok(())
                }
           }
            impl toql::tree::tree_identity::TreeIdentity for &mut #struct_name_ident {
             #[allow(unused_mut)]
             fn auto_id< 'a, I >( mut descendents : I) -> std :: result :: Result < bool, toql::error::ToqlError >
             where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
             {
                 <#struct_name_ident as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
             }
              #[allow(unused_mut)]
             fn set_id < 'a, 'b, I >(&mut self, mut descendents : I, action: &'b toql::tree::tree_identity::IdentityAction)
                        -> std::result::Result<(),toql::error::ToqlError>
                 where I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone
            {
                 toql::tree::tree_identity::TreeIdentity::set_id(*self, descendents, action)
            }
            }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
