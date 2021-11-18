use crate::parsed::{
    field::{
        field_kind::FieldKind,
        regular_field::{RegularSelection, SqlTarget},
    },
    parsed_struct::ParsedStruct,
};
use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut key_fields_code = Vec::new();
    let mut key_columns_code = Vec::new();
    let mut foreign_key_columns_code = Vec::new();
    let mut foreign_key_set_column_code = Vec::new();
    let mut key_inverse_columns_code = Vec::new();
    let mut key_params_code = Vec::new();
    let mut key_types = Vec::new();
    let mut key_fields = Vec::new();
    let mut key_setters = Vec::new();
    let mut key_getters = Vec::new();
    let mut key_field_declarations = Vec::new();
    let mut toql_eq_predicates = Vec::new();
    let mut toql_eq_foreign_predicates = Vec::new();
    let mut key_constr_code = Vec::new();
    let mut sql_arg_code = None;
    let mut try_from_setters = Vec::new();

    let struct_key_ident = Ident::new(
        &format!("{}Key", &parsed_struct.struct_name),
        Span::call_site(),
    );

    for field in &parsed_struct.fields {
        let field_type = &field.field_type;
        let field_base_type = &field.field_base_type;
        let field_name_ident = &field.field_name;
        let field_name = &field.field_name.to_string();
        let toql_query_name = &field.toql_query_name;

        match &field.kind {
            FieldKind::Regular(ref regular_attrs) => {
                if regular_attrs.foreign_key || regular_attrs.key {
                    if let SqlTarget::Column(ref column) = &regular_attrs.sql_target {
                        foreign_key_columns_code
                            .push(quote!( columns.push( String::from(#column)); ));

                        let setter_code = match &regular_attrs.selection {
                            RegularSelection::SelectNullable => {
                                quote!(self. #field_name_ident = Some(if value.is_null(){ None}else {Some(value.try_into()?)}))
                            }
                            RegularSelection::PreselectNullable | RegularSelection::Select => {
                                quote!(self. #field_name_ident = if value.is_null(){ None}else {Some(value.try_into()?)})
                            }
                            RegularSelection::Preselect => {
                                quote!(self. #field_name_ident = value.try_into()?)
                            }
                        };

                        foreign_key_set_column_code.push(quote!( #column =>  #setter_code, ));
                    }
                }

                if !regular_attrs.key {
                    continue;
                }

                if let SqlTarget::Column(ref column) = &regular_attrs.sql_target {
                    key_columns_code.push(quote!( columns.push( String::from(#column)); ));

                    key_fields_code.push(quote!( fields.push( String::from(#toql_query_name)); ));

                    if let Some(inverse_column) = &regular_attrs.default_inverse_column {
                        key_inverse_columns_code
                            .push(quote!( columns.push( String::from(#inverse_column)); ));
                    }

                    toql_eq_predicates.push( quote!(.and(toql::query::field::Field::from(#toql_query_name).eq(&t. #field_name_ident))));
                    toql_eq_foreign_predicates.push( quote!(.and( toql::query::field::Field::from(format!("{}_{}",toql_path ,#toql_query_name)).eq(&t.#field_name_ident))));

                    key_constr_code.push(quote!(#field_name_ident));

                    key_field_declarations.push(quote!( pub #field_name_ident: #field_type));

                    sql_arg_code = if key_columns_code.len() == 1 {
                        Some(quote!(
                            impl From< #struct_key_ident> for toql::sql_arg::SqlArg {
                                fn from( t: #struct_key_ident) -> toql::sql_arg::SqlArg {
                                    toql::sql_arg::SqlArg::from(t. #field_name_ident)
                                }
                            }
                            impl From<&#struct_key_ident> for toql::sql_arg::SqlArg {
                                fn from(t: &#struct_key_ident) -> toql::sql_arg::SqlArg {
                                    toql::sql_arg::SqlArg::from(t. #field_name_ident .to_owned())
                                }
                            }
                        ))
                    } else {
                        None
                    };
                }

                key_types.push(quote!( #field_base_type));
                key_fields.push(quote!(self. #field_name_ident .to_owned()));
                key_getters.push(quote!(#field_name_ident : self. #field_name_ident .to_owned()));
                key_setters.push(quote!(self. #field_name_ident = key . #field_name_ident));

                let try_from_setters_index = syn::Index::from(try_from_setters.len());
                try_from_setters
                    .push(quote!( #field_name_ident : args
                                .get(#try_from_setters_index)
                                .ok_or(toql::error::ToqlError::ValueMissing( #field_name.to_string()))?
                                .try_into()?)); // Better Error

                key_params_code.push(
                    quote!(params.push(toql::sql_arg::SqlArg::from(&key . #field_name_ident)); ),
                );
            }
            FieldKind::Join(ref join_attrs) => {
                if !join_attrs.key {
                    continue;
                }
                let default_self_column_code = &join_attrs.default_self_column_code;

                key_types.push(quote!( <#field_base_type as toql::keyed::Keyed>::Key));

                let toql_name = &field.toql_query_name;
                toql_eq_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #field_name_ident, #toql_name))));
                toql_eq_foreign_predicates.push(quote!(.and(toql::to_query::ToForeignQuery::to_foreign_query::<_>(&t. #field_name_ident, #toql_name))));

                key_constr_code.push(quote!(#field_name_ident));

                key_field_declarations
                    .push(quote!( pub #field_name_ident: <#field_type as toql::keyed::Keyed>::Key));

                let columns_map_code = &join_attrs.columns_map_code;
                key_columns_code.push(quote!(

                <<#field_type as toql::keyed::Keyed>::Key as toql::key::Key>::columns().iter().for_each(|other_column| {
                    #default_self_column_code;
                    let column = #columns_map_code;
                    columns.push(column.to_string());
                });
                ));

                key_fields_code.push(quote!(
                 <<#field_type as toql::keyed::Keyed>::Key as toql :: key_fields :: KeyFields> :: fields().iter().for_each(|other_field| {
                    fields.push(format!("{}_{}",#toql_query_name, other_field));
                });
                ));

                key_inverse_columns_code.push( quote!(

                        <<#field_type as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns().iter().for_each(|other_column| {
                            #default_self_column_code;
                            let column = #columns_map_code;
                            columns.push(column.to_string());
                        });
                        ));

                key_params_code.push( quote!( params.extend_from_slice(&toql::key::Key::params(& key. #field_name_ident)); ));

                key_fields.push(quote!(
                    < #field_type as toql::keyed::Keyed>::key(  &self. #field_name_ident )
                ));
                key_getters.push(quote!(#field_name_ident :
                    < #field_type as toql::keyed::Keyed>::key(  &self. #field_name_ident )
                ));

                key_setters.push( quote!(
                                    < #field_type as toql::keyed::KeyedMut>::set_key(&mut self. #field_name_ident,key . #field_name_ident)
                            ));

                let try_from_setters_index = syn::Index::from(try_from_setters.len());
                try_from_setters
                .push(quote!( #field_name_ident :
                                        <#field_type as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[ #try_from_setters_index..]))?
             ));
            }
            _ => {}
        }
    }

    // Generate token stream
    let key_type_arg = if key_types.len() == 1 {
        quote!( #(#key_types),* )
    } else {
        quote!( ( #( #key_types),*) )
    };

    let key_field_declarations = &key_field_declarations;

    let serde = if cfg!(feature = "serde") {
        quote!(toql::serde::Serialize, toql::serde::Deserialize,)
    } else {
        quote!()
    };

    let key_constr_code = if key_constr_code.len() == 1 {
        let key_constr_code = key_constr_code.get(0).unwrap();
        vec![quote!( #key_constr_code : key)]
    } else {
        key_constr_code
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let index = syn::Index::from(i);
                quote!(#k: key. #index)
            })
            .collect::<Vec<_>>()
    };

    let struct_name_ident = &parsed_struct.struct_name;
    let vis = &parsed_struct.vis;
    let number_of_foreign_columns = foreign_key_columns_code.len();

    let key = quote! {
        #[derive(Debug, Eq, PartialEq, Hash, #serde Clone)]

        #vis struct #struct_key_ident {
            #(#key_field_declarations),*
        }

        impl toql::key_fields::KeyFields  for #struct_key_ident {
            type Entity = #struct_name_ident;

            fn fields() ->Vec<String> {
                let mut fields: Vec<String>= Vec::new();

                #(#key_fields_code)*
                fields
            }

            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                let mut params: Vec<toql::sql_arg::SqlArg>= Vec::new();
                let key = self; // TODO cleanup

                #(#key_params_code)*
                params
            }
        }
        impl toql::key_fields::KeyFields  for &#struct_key_ident {
            type Entity = #struct_name_ident;

            fn fields() ->Vec<String> {
                <#struct_key_ident as toql::key_fields::KeyFields>::fields()
            }

            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                <#struct_key_ident as toql::key_fields::KeyFields>::params(self)
            }
        }

        impl toql::key::Key  for #struct_key_ident {
            type Entity = #struct_name_ident;

            fn columns() ->Vec<String> {
                let mut columns: Vec<String>= Vec::new();

                #(#key_columns_code)*
                columns
            }
            fn default_inverse_columns() ->Vec<String> {
                let mut columns: Vec<String>= Vec::new();

                #(#key_inverse_columns_code)*
                columns
            }
            fn params(&self) ->Vec<toql::sql_arg::SqlArg> {
                let mut params: Vec<toql::sql_arg::SqlArg>= Vec::new();
                let key = self; // TODO cleanup

                #(#key_params_code)*
                params
            }
        }

        impl toql::key::Key for &#struct_key_ident {
            type Entity = #struct_name_ident;
            fn columns() -> Vec<String> {
                <#struct_key_ident as toql::key::Key>::columns()
            }
            fn default_inverse_columns() -> Vec<String> {
            <#struct_key_ident as toql::key::Key>::default_inverse_columns()
            }
            fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
                <#struct_key_ident as toql::key::Key>::params(self)
            }
        }

        #sql_arg_code

        impl toql::keyed::Keyed for #struct_name_ident {
            type Key = #struct_key_ident;

            fn key(&self) -> Self::Key {
                #struct_key_ident {
                   #( #key_getters),*
                   }
            }
        }

         impl toql::keyed::Keyed for &#struct_name_ident {
            type Key = #struct_key_ident;

            fn key(&self) -> Self::Key {
                <#struct_name_ident as toql::keyed::Keyed>::key(self)
            }
        }
        impl toql::keyed::Keyed for &mut #struct_name_ident {
            type Key = #struct_key_ident;

            fn key(&self) -> Self::Key {
                 <#struct_name_ident as toql::keyed::Keyed>::key(self)
            }
        }



        impl toql::keyed::KeyedMut for #struct_name_ident {

            fn set_key(&mut self, key: Self::Key)  {
              #( #key_setters;)*
            }

        }

         impl toql::keyed::KeyedMut for &mut #struct_name_ident {

            fn set_key(&mut self, key: Self::Key)  {
                <#struct_name_ident as toql::keyed::KeyedMut>::set_key(self, key)
            }

        }

        impl std::convert::TryFrom<Vec< toql::sql_arg::SqlArg>> for #struct_key_ident
        {
            type Error = toql::error::ToqlError;
            fn try_from(args: Vec< toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
                use std::convert::TryInto;

               Ok(#struct_key_ident {
                   #( #try_from_setters),*
               })

            }
        }

        impl std::convert::From<#key_type_arg> for #struct_key_ident
        {

            fn from(key: #key_type_arg) ->Self {
                Self{
                    #(#key_constr_code),*
                }
            }
        }


        // Impl to support HashSets
        impl std::hash::Hash for #struct_name_ident {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                <#struct_name_ident as toql::keyed::Keyed>::key(self).hash(state);
            }
        }

        impl toql::identity::Identity for #struct_name_ident {
            fn columns() -> Vec<String> {
                let mut columns = Vec::with_capacity(#number_of_foreign_columns);
                #(#foreign_key_columns_code)*
                columns

            }
            fn set_column(&mut self, column:&str, value: &toql::sql_arg::SqlArg) -> toql::result::Result<()> {
                use std::convert::TryInto;
                match column {
                     #(#foreign_key_set_column_code)*
                     _ => {}

                }
                Ok(())

            }

        }

    };

    log::debug!(
        "Source code for `{}`:\n{}",
        struct_name_ident,
        key.to_string()
    );
    tokens.extend(key);
}
