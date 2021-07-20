/*
* Generation functions for toql derive
*
*/

use crate::sane::{FieldKind, MergeColumn, Struct};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::Ident;

pub(crate) struct CodegenTree<'a> {
    rust_struct: &'a Struct,

    dispatch_predicate_args_code: Vec<TokenStream>,
    dispatch_predicate_columns_code: Vec<TokenStream>,

    merge_columns_code: Vec<TokenStream>,
    merge_predicate_code: Vec<TokenStream>,

    index_type_bounds: Vec<TokenStream>,
    dispatch_index_code: Vec<TokenStream>,
    index_code: Vec<TokenStream>,
    merge_type_bounds: Vec<TokenStream>,
    dispatch_merge_code: Vec<TokenStream>,
    merge_code: Vec<TokenStream>,

    dispatch_auto_id_code: Vec<TokenStream>,
    dispatch_identity_code: Vec<TokenStream>,
    identity_set_merges_key_code: Vec<TokenStream>,
    key_columns: Vec<String>,

    dispatch_map_code: Vec<TokenStream>,
    dispatch_types: HashSet<Ident>,

    number_of_keys: u8,
}

impl<'a> CodegenTree<'a> {
    pub(crate) fn from_toql(toql: &crate::sane::Struct) -> CodegenTree {
        CodegenTree {
            rust_struct: &toql,

            dispatch_predicate_args_code: Vec::new(),
            dispatch_predicate_columns_code: Vec::new(),

            merge_columns_code: Vec::new(),
            merge_predicate_code: Vec::new(),

            index_type_bounds: Vec::new(),
            dispatch_index_code: Vec::new(),
            index_code: Vec::new(),
            merge_type_bounds: Vec::new(),
            dispatch_merge_code: Vec::new(),
            merge_code: Vec::new(),

            dispatch_auto_id_code: Vec::new(),
            dispatch_identity_code: Vec::new(),
            identity_set_merges_key_code: Vec::new(),
            key_columns: Vec::new(),
            number_of_keys: 0,
            dispatch_map_code: Vec::new(),
            dispatch_types: HashSet::new(),
        }
    }

    pub(crate) fn add_tree_traits(&mut self, field: &crate::sane::Field) {
        use crate::sane::SqlTarget;

        let rust_field_ident = &field.rust_field_ident;
        let rust_field_name = &field.rust_field_name;
        let rust_type_ident = &field.rust_type_ident;
        let rust_type_name = &field.rust_type_name;
        let toql_field_name = &field.toql_field_name;
        let rust_base_type_ident = &field.rust_base_type_ident;

        // Handle key predicate and parameters
        let unwrap = match field.number_of_options {
            1 => {
                quote!(.as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
            0 => quote!(),
            _ => {
                quote!(.as_ref().unwrap().as_ref().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
        };

        let refer = match field.number_of_options {
            0 => quote!(&),
            _ => quote!(),
        };
        let unwrap_mut = match field.number_of_options {
            1 => {
                quote!(.as_mut().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
            0 => quote!(),
            _ => {
                quote!(.as_mut().unwrap().as_mut().ok_or(toql::error::ToqlError::ValueMissing(#rust_field_name.to_string()))?)
            }
        };

        let refer_mut = match field.number_of_options {
            0 => quote!(&mut),
            _ => quote!(),
        };

        match &field.kind {
            FieldKind::Regular(field_attrs) => {
                if field_attrs.key {
                    self.number_of_keys += 1;
                    if let SqlTarget::Column(column) = &field_attrs.sql_target {
                        self.key_columns.push(column.to_string())
                    }
                }
            }
            FieldKind::Join(join_attrs) => {
                if join_attrs.key {
                    self.number_of_keys += 1;
                }

                self.dispatch_types
                    .insert(field.rust_base_type_ident.to_owned());

                self.dispatch_map_code.push(quote!(
                            <#rust_base_type_ident as toql::tree::tree_map::TreeMap>::map(registry)?;
                ));

                self.dispatch_predicate_args_code.push(quote!(
                      #toql_field_name => {
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            args(#refer  self. #rust_field_ident # unwrap ,&mut descendents, args)?
                        }
                ));
                self.dispatch_predicate_columns_code.push(quote!(
                      #toql_field_name => {
                            <#rust_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            columns(#refer  self. #rust_field_ident # unwrap ,&mut descendents)?
                        }
                ));

                self.dispatch_index_code.push(quote!(
                        #toql_field_name => {
                            <#rust_type_ident as toql::tree::tree_index::TreeIndex<R,E>>::
                            index(&mut descendents, rows, row_offset, index)?
                        }
                ));
                self.index_type_bounds.push(quote!(
                    #rust_type_ident : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

                self.dispatch_merge_code.push(
                   quote!(
                       #toql_field_name => {

                            <#rust_type_ident as toql::tree::tree_merge::TreeMerge<R, E>>::
                            merge(#refer_mut self. #rust_field_ident #unwrap_mut, &mut descendents, &field, rows, row_offset, index, selection_stream)?

                       }
                )
               );
                self.merge_type_bounds.push(quote!(
                    #rust_type_ident : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

                self.dispatch_identity_code.push(
                   quote!(
                       #toql_field_name => {
                            <#rust_type_ident as toql::tree::tree_identity::TreeIdentity>::
                            set_id(#refer_mut self. #rust_field_ident #unwrap_mut, &mut descendents, action)?
                        }
                )
               );
                self.dispatch_auto_id_code.push(
                   quote!(
                       #toql_field_name => {
                            Ok(<#rust_type_ident as toql::tree::tree_identity::TreeIdentity>::auto_id(&mut descendents)?)
                        }
                )
               );

                if join_attrs.skip_mut_self_cols {
                    self.identity_set_merges_key_code.push(
                        quote!(
                             if let Some(mut f) = self. #rust_field_ident .as_mut() {
                        // Get INverse columns
                         let default_inverse_columns = << QuestionAnswer as toql :: keyed ::   Keyed > :: Key as toql :: key :: Key > ::
                        default_inverse_columns() ;

                        let inverse_columns =
                            <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    let inverse_column = match c.as_str() {
                                        _ => default_inverse_columns.get(i).unwrap().to_owned(),
                                    };
                                    inverse_column
                                })
                                .collect::<Vec<String>>();
                       
                        // Build target key args
                        let mut args = Vec::new();
                        for c in inverse_columns {
                            let i = key_columns.iter().position(|r| r == &c)
                                .ok_or_else(|| toql::sql_mapper::SqlMapperError::ColumnMissing(#rust_type_name.to_string(), c.to_string()))?;
                            args.push(self_key_args.get(i).ok_or_else(||toql::sql_mapper::SqlMapperError::ColumnMissing(#rust_type_name.to_string(), c.to_string()))?.to_owned());
                        }

                            

                        let action = toql::tree::tree_identity::IdentityAction::Set(std::cell::RefCell::new(args));
                        <#rust_type_ident as toql::tree::tree_identity::TreeIdentity>::set_id(f, &mut std::iter::empty(), &action)?;
                             }
                        )
                    ); 

                }


            }
            FieldKind::Merge(merge) => {
                self.dispatch_types
                    .insert(field.rust_base_type_ident.to_owned());

                self.dispatch_index_code.push(quote!(
                       #toql_field_name => {
                             <#rust_base_type_ident as toql::tree::tree_index::TreeIndex<R,E>>::
                            index(&mut descendents, rows, row_offset, index)?
                       }
                ));
                self.dispatch_merge_code.push(
                   quote!(
                       #toql_field_name => {
                        for f in #refer_mut self. #rust_field_ident #unwrap_mut {
                            <#rust_base_type_ident as toql::tree::tree_merge::TreeMerge<R,E>>::
                            merge(f, &mut descendents, &field, rows, row_offset, index, selection_stream)?
                        }
                       }
                )
               );

                self.dispatch_predicate_args_code.push(quote!(
                       #toql_field_name => {
                        for f in #refer self. #rust_field_ident #unwrap {
                            <#rust_base_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            args(f, &mut descendents, args)?
                        }
                       }
                ));
                self.dispatch_predicate_columns_code.push(
                   quote!(
                       #toql_field_name => {
                       let f = #refer self. #rust_field_ident #unwrap .get(0)
                        .ok_or(
                            toql::error::ToqlError::SqlBuilderError(
                             toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(#toql_field_name.to_string())))
                             ?;
                            <#rust_base_type_ident as toql::tree::tree_predicate::TreePredicate>::
                            columns(f, &mut descendents)?


                       }
                )
               );

                self.dispatch_map_code.push(quote!(
                            <#rust_base_type_ident as toql::tree::tree_map::TreeMap>::map(registry)?;
                ));

                /*  self.dispatch_merge_key_code.push(quote!(
                       #toql_field_name => {
                            <#rust_base_type_ident as toql::tree::tree_keys::TreeKeys>::
                            keys(&mut descendents, field, key_expr)?
                        }
                )); */
                self.dispatch_auto_id_code.push(quote!(
                       #toql_field_name => {
                                Ok(<#rust_base_type_ident as toql::tree::tree_identity::TreeIdentity>::
                                auto_id(&mut descendents)?)
                        }
                ));
                self.dispatch_identity_code.push(quote!(
                       #toql_field_name => {
                            for f in #refer_mut self. #rust_field_ident #unwrap_mut {
                                <#rust_base_type_ident as toql::tree::tree_identity::TreeIdentity>::
                                set_id(f, &mut descendents,  action.clone())?
                            }
                        }
                ));
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
                self.merge_columns_code.push(quote!(
                       #toql_field_name => {
                            // Primary key
                            /* for col in <<#rust_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns() {
                                key_expr.push_self_alias();
                                key_expr.push_literal(".");
                                key_expr.push_alias(col);
                                key_expr.push_literal(", ");
                            } */
                           #(#columns_merge)*
                       }
                ));

                let mut columns_code: Vec<TokenStream> = Vec::new();

                for c in &merge.columns {
                    columns_code.push(match &c.other {
                            MergeColumn::Aliased(a) => { quote!( columns.push(  toql :: sql_expr :: PredicateColumn::Literal(#a .to_owned())); )}
                            MergeColumn::Unaliased(a) => {quote!( columns.push(  toql :: sql_expr :: PredicateColumn::SelfAliased(#a .to_owned())); )}
                        });
                }

                self.merge_predicate_code.push(
                   quote!(
                       #toql_field_name => {
                                let key = < Self as toql :: keyed :: Keyed >::key(&self);
                                let params =<< Self as toql :: keyed :: Keyed > :: Key as toql :: key :: Key > ::params(&key);
                                let mut columns :Vec<toql::sql_expr::PredicateColumn> = Vec::new();
                                #(#columns_code)*
                                predicate.push_predicate( columns, params);
                        },

                )
               );

                /*   let type_key_ident =
                Ident::new(&format!("{}Key", &field.rust_type_name), Span::call_site()); */

                let struct_ident = &self.rust_struct.rust_struct_ident;

                let struct_key_ident =
                    Ident::new(&format!("{}Key", &struct_ident), Span::call_site());

                self.index_type_bounds.push(quote!(
                    //#type_key_ident : toql :: from_row :: FromRow < R >,
                    <#rust_type_ident as toql::keyed::Keyed>::Key : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < <#rust_type_ident as toql::keyed::Keyed>::Key as toql :: from_row :: FromRow < R >> :: Error>
                    ));
                self.merge_type_bounds.push(quote!(
                    <#rust_type_ident as toql::keyed::Keyed>::Key : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < <#rust_type_ident as toql::keyed::Keyed>::Key as toql :: from_row :: FromRow < R >> :: Error>,
                    #rust_type_ident : toql :: from_row :: FromRow < R >,
                    E : std::convert::From< < #rust_type_ident as toql :: from_row :: FromRow < R >> :: Error>
                    ));

                self.index_code.push(quote!(
                    #toql_field_name => {
                        let fk = <#rust_type_ident as toql::keyed::Keyed>::Key ::from_row(&row, &mut i, &mut iter)?;
                        fk.hash(&mut s);
                        },
                ));
                let merge_push = if field.number_of_options > 0 {
                    quote!( if self. #rust_field_ident .is_none() {
                            self. #rust_field_ident = Some(Vec::new())};
                            self. #rust_field_ident .as_mut().unwrap() .push(e);
                    )
                } else {
                    quote!(self. #rust_field_ident .push(e);)
                };

                self.merge_code.push(
                    quote!(

                             #toql_field_name  => {
                                for row_number in row_numbers {
                                            let mut i = n;
                                            let mut iter = std::iter::repeat(&Select::Query);
                                            let row: &R = &rows[*row_number];
                                            let fk = #struct_key_ident::from_row(&row, &mut i, &mut iter)?
                                                .ok_or(toql::error::ToqlError::ValueMissing( #toql_field_name .to_string()))?;
                                            if fk ==  pk {
                                              //  let mut i = 0;
                                                let mut iter = selection_stream.iter();
                                                let e = #rust_base_type_ident::from_row(&row, &mut i, &mut iter)?
                                                  .ok_or(toql::error::ToqlError::ValueMissing( #toql_field_name .to_string()))?;
                                                #merge_push
                                            }
                                        }
                                },
                    )
                );

                let mut skip_identity_code = false;
                let mut column_mapping = Vec::new();

                if merge.columns.is_empty() {
                    let rust_struct_name = &self.rust_struct.rust_struct_name;

                    for this_column in &self.key_columns {
                        let other_column = format!(
                            "{}_{}",
                            &heck::SnakeCase::to_snake_case(rust_struct_name.as_str()),
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
                    match field.number_of_options {
                         0 => {self.identity_set_merges_key_code.push(
                            quote!(
                                let self_columns = <#struct_key_ident as toql::key::Key>::columns();
                                for e in #refer self. #rust_field_ident #unwrap_mut {
                                    let key = < #rust_type_ident as toql :: keyed :: Keyed >::key(e);
                                            let mut ps = toql::key::Key::params(&key);

                                            let other_columns = <<#rust_base_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                                            for (i,other_column) in other_columns.iter().enumerate() {
                                                for (j,self_column) in self_columns.iter().enumerate() {
                                                    let calculated_other_column= match self_column.as_str() {
                                                        #(#column_mapping,)*
                                                        x @ _ => x
                                                    };
                                                    if other_column == &calculated_other_column {
                                                        let p = ps.get_mut(i).unwrap();
                                                        *p = self_key_params.get(j).unwrap().to_owned();
                                                    }
                                                }
                                            }
                                            let key = <<#rust_base_type_ident as toql::keyed::Keyed>::Key as std::convert::TryFrom<_>>::try_from(ps)?;
                                            toql::keyed::KeyedMut::set_key(e, key);
                                }
                            )
                        )  },
                        _ => {
                             self.identity_set_merges_key_code.push( quote!(
                                  let self_columns = <#struct_key_ident as toql::key::Key>::columns();
                                    if let Some(u) = self. #rust_field_ident .as_mut() {
                                        for e in u {
                                            let key = toql::keyed::Keyed::key(e);
                                            let mut ps = toql::key::Key::params(&key);

                                            let other_columns = <<#rust_base_type_ident as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                                            for (i,other_column) in other_columns.iter().enumerate() {
                                                for (j,self_column) in self_columns.iter().enumerate() {
                                                    let calculated_other_column= match self_column.as_str() {
                                                        #(#column_mapping,)*
                                                        x @ _ => x
                                                    };
                                                    if other_column == &calculated_other_column {
                                                        let p = ps.get_mut(i).unwrap();
                                                        *p = self_key_params.get(j).unwrap().to_owned();
                                                    }
                                                }
                                            }
                                            let key = <<#rust_base_type_ident as toql::keyed::Keyed>::Key as std::convert::TryFrom<_>>::try_from(ps)?;
                                            toql::keyed::KeyedMut::set_key(e, key);
                                        }

                                    }))
                        }
                        }
                }
            }
        };
    }
}
impl<'a> quote::ToTokens for CodegenTree<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_ident = &self.rust_struct.rust_struct_ident;
        let struct_name = &self.rust_struct.rust_struct_name;

        let dispatch_predicate_args_code = &self.dispatch_predicate_args_code;
        let dispatch_predicate_columns_code = &self.dispatch_predicate_columns_code;

        let dispatch_index_code = &self.dispatch_index_code;

        let dispatch_merge_code = &self.dispatch_merge_code;
        let merge_code = &self.merge_code;

        let dispatch_auto_id_code = &self.dispatch_auto_id_code;
        let dispatch_identity_code = &self.dispatch_identity_code;
        let identity_set_merges_key_code = &self.identity_set_merges_key_code;

        let dispatch_map_code = &self.dispatch_map_code;

        let struct_key_ident = Ident::new(
            &format!("{}Key", &self.rust_struct.rust_struct_ident),
            Span::call_site(),
        );

        let identity_set_self_key_code = quote!(
            if let toql::tree::tree_identity::IdentityAction::Set(args) = action {
                let n = <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
                let end = args.borrow().len();
                let args: Vec<toql::sql_arg::SqlArg> =
                    args.borrow_mut().drain(end - n..).collect::<Vec<_>>();
                let key = std::convert::TryFrom::try_from(args)?;

                <Self as toql::keyed::KeyedMut>::set_key(self, key);
            }
        );

        let identity_set_key = if self.rust_struct.auto_key {
            quote!( #identity_set_self_key_code

                    let self_key = <Self as toql::keyed::Keyed>::key(&self);
                    let self_key_params = toql::key::Key::params(&self_key);
                    let key_columns =  <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();

                   #(#identity_set_merges_key_code)*)
        } else {
            quote!()
        };

        let identity_auto_id_code = if self.number_of_keys == 1 && self.rust_struct.auto_key {
            quote!(true)
        } else {
            quote!(false)
        };

        let tree_index_dispatch_bounds = self
            .dispatch_types
            .iter()
            .map(|t| quote!( #t :  toql::tree::tree_index::TreeIndex<R, E>,))
            .collect::<Vec<_>>();
        let tree_index_dispatch_bounds_ref = tree_index_dispatch_bounds.clone();

        let tree_merge_dispatch_bounds = self
            .dispatch_types
            .iter()
            .map(|t| {
                quote!(
                #t : toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
                )
            })
            .collect::<Vec<_>>();
        let tree_merge_dispatch_bounds_ref = tree_merge_dispatch_bounds.clone();

        let mods = quote! {

                impl toql::tree::tree_identity::TreeIdentity for #struct_ident {
                #[allow(unused_mut)]
                 fn auto_id< 'a, I >(mut descendents : & mut I) -> std :: result :: Result < bool, toql::error::ToqlError >
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
                 fn set_id < 'a, 'b, I >(&mut self, mut descendents : & mut I, action: &'b toql::tree::tree_identity::IdentityAction)
                            -> std :: result :: Result < (), toql::error::ToqlError >
                            where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
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
                impl toql::tree::tree_identity::TreeIdentity for &mut #struct_ident {
                 #[allow(unused_mut)]
                 fn auto_id< 'a, I >( mut descendents : & mut I) -> std :: result :: Result < bool, toql::error::ToqlError > 
                 where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                 {
                     <#struct_ident as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
                 }
                  #[allow(unused_mut)]
                 fn set_id < 'a, 'b, I >(&mut self, mut descendents : & mut I, action: &'b toql::tree::tree_identity::IdentityAction)
                            -> std::result::Result<(),toql::error::ToqlError>
                     where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                {
                     <#struct_ident as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
                }
                }
                 impl toql::tree::tree_map::TreeMap for #struct_ident {

                         fn map(registry: &mut toql::sql_mapper_registry::SqlMapperRegistry)-> toql::result::Result<()>{

                                 if registry.get(#struct_name).is_none() {
                                     registry.insert_new_mapper::<#struct_ident>()?;
                                 }
                                 #(#dispatch_map_code)*
                                 Ok(())
                         }

                 }
                  impl toql::tree::tree_map::TreeMap for &#struct_ident {

                         fn map(registry: &mut toql::sql_mapper_registry::SqlMapperRegistry)-> toql::result::Result<()>{
                             <#struct_ident as  toql::tree::tree_map::TreeMap>::map(registry)
                         }
                  }

                impl toql::tree::tree_predicate::TreePredicate for #struct_ident {

                     #[allow(unused_mut)]
                    fn columns<'a, I>(&self, mut descendents: &mut I )
                        -> std::result::Result<Vec<String>, toql::error::ToqlError>
                        where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                        {
                        Ok(match descendents.next() {
                                Some(d) => match d.as_str() {
                                    #(#dispatch_predicate_columns_code),*
                                    f @ _ => {
                                            return Err(
                                                toql::error::ToqlError::SqlBuilderError (
                                                 toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string())));
                                        }
                                },
                                None => {
                                   <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns()

                                    /*
                                        match field {
                                        #(#merge_predicate_code),*
                                        f @ _ => {
                                            return Err(
                                                toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                        }
                                        }*/

                                }
                            })
                        }

                     #[allow(unused_mut)]
                    fn args<'a, I>(
                        &self,
                        mut descendents: &mut I,
                        args: &mut Vec<toql::sql_arg::SqlArg>
                    ) -> std::result::Result<(), toql::error::ToqlError>
                     where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                    {
                            match descendents.next() {
                                Some(d) => match d.as_str() {
                                    #(#dispatch_predicate_args_code),*
                                    f @ _ => {
                                            return Err(
                                                toql::error::ToqlError::SqlBuilderError (
                                                 toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string())));
                                        }
                                },
                                None => {
                                     let key = <Self as toql::keyed::Keyed>::key(&self);
                                    args.extend(<<Self as toql::keyed::Keyed>::Key as toql::key::Key>::params(&key));
                               }
                            }
                            Ok(())
                        }
               }

                impl toql::tree::tree_predicate::TreePredicate for &#struct_ident {

                     #[allow(unused_mut)]
                    fn columns<'a, I>(&self, mut descendents: &mut I )
                        -> std::result::Result<Vec<String>, toql::error::ToqlError>
                        where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                        {
                            <#struct_ident as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
                        }

                     #[allow(unused_mut)]
                     fn args<'a, I>(
                        &self,
                        mut descendents: &mut I,
                        args: &mut Vec<toql::sql_arg::SqlArg>
                    ) -> std::result::Result<(), toql::error::ToqlError>
                     where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                    {
                        <#struct_ident as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
                    }
                }
                impl toql::tree::tree_predicate::TreePredicate for &mut #struct_ident {

                    #[allow(unused_mut)]
                    fn columns<'a, I>(&self, mut descendents: &mut I )
                        -> std::result::Result<Vec<String>, toql::error::ToqlError>
                        where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                        {
                            <#struct_ident as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
                        }

                     #[allow(unused_mut)]
                     fn args<'a, I>(
                        &self,
                        mut descendents: &mut I,
                        args: &mut Vec<toql::sql_arg::SqlArg>
                    ) -> std::result::Result<(), toql::error::ToqlError>
                     where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                    {
                        <#struct_ident as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
                    }
                }




                 impl<R,E> toql::tree::tree_index::TreeIndex<R, E> for #struct_ident
                  where  E: std::convert::From<toql::error::ToqlError>,
                  #struct_key_ident: toql::from_row::FromRow<R, E>,
                  #(#tree_index_dispatch_bounds)*
               /*  where Self: toql::from_row::FromRow<R>,
                #struct_key_ident : toql :: from_row :: FromRow < R >,
                E : std::convert::From< <#struct_key_ident as toql :: from_row :: FromRow < R >> :: Error>,
                E: std::convert ::From<toql :: sql_builder :: sql_builder_error ::  SqlBuilderError>, */
             //   #(#index_type_bounds)*

                {
                     #[allow(unused_variables, unused_mut)]
                    fn index<'a, I>( mut descendents: &mut I,
                                rows: &[R], row_offset: usize, index: &mut std::collections::HashMap<u64,Vec<usize>>)
                        -> std::result::Result<(), E>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                         {

                        use toql::from_row::FromRow;
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        use std::collections::hash_map::DefaultHasher;
                         use toql::sql_builder::select_stream::Select;

                      match descendents.next() {

                                Some(d) => {
                                    match d.as_str() {
                                        #(#dispatch_index_code),*
                                        f @ _ => {
                                            return Err(
                                                toql::error::ToqlError::SqlBuilderError (
                                                 toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                                 .into());
                                        }
                                    }
                                },
                                None => {


                                        for (n, row) in rows.into_iter().enumerate() {
                                            let mut iter = std::iter::repeat(&Select::Query);
                                            let mut  i= row_offset;
                                            let fk = #struct_key_ident ::from_row(&row, &mut i, &mut iter)?
                                             .ok_or(toql::error::ToqlError::ValueMissing(
                                                             <#struct_key_ident as toql::key::Key>::columns().join(", ")
                                                         ))?; // SKip Primary key

                                            let mut s = DefaultHasher::new();
                                            fk.hash(&mut s);
                                           /*  match field {
                                               #(#index_code)*

                                                f @ _ => {
                                                    return Err(
                                                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()).into());
                                                }

                                            }; */
                                            let fk_hash =  s.finish();

                                            index.entry(fk_hash)
                                            .and_modify(|h| h.push(n))
                                            .or_insert(vec![n]);
                                        }



                                    }



                        }
                        Ok(())
                    }

                }


              impl<R,E> toql::tree::tree_index::TreeIndex<R, E> for &#struct_ident
                  where  E: std::convert::From<toql::error::ToqlError>,
                  #struct_key_ident: toql::from_row::FromRow<R, E>,
                  #(#tree_index_dispatch_bounds_ref)*
               {
                    #[allow(unused_mut)]
                    fn index<'a, I>( mut descendents: &mut I,
                                rows: &[R], row_offset: usize, index: &mut std::collections::HashMap<u64,Vec<usize>>)
                        -> std::result::Result<(), E>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                         {
                             <#struct_ident as  toql::tree::tree_index::TreeIndex<R,E>>::index(descendents,  rows, row_offset, index)
                         }
                }



                impl<R,E> toql::tree::tree_merge::TreeMerge<R,E> for #struct_ident
                 where  E: std::convert::From<toql::error::ToqlError>,
                 #struct_key_ident: toql::from_row::FromRow<R, E>,
                 #(#tree_merge_dispatch_bounds)*


                {
                    #[allow(unreachable_code, unused_variables, unused_mut)]
                    fn merge<'a, I>(  &mut self, mut descendents: &mut I, field: &str,
                                rows: &[R],row_offset: usize, index: &std::collections::HashMap<u64,Vec<usize>>, selection_stream: &toql::sql_builder::select_stream::SelectStream)
                        -> std::result::Result<(), E>
                        where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                         {
                        use toql::keyed::Keyed;
                        use toql::from_row::FromRow;
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        use std::collections::hash_map::DefaultHasher;
                        use toql::sql_builder::select_stream::Select;

                      match descendents.next() {

                                Some(d) => {
                                    match d.as_str() {
                                        #(#dispatch_merge_code),*
                                        f @ _ => {
                                            return Err(
                                               toql::error::ToqlError::SqlBuilderError(
                                                   toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                                   .into());
                                        }
                                    }
                                },
                                None => {

                                        let pk : #struct_key_ident = <Self as toql::keyed::Keyed>::key(&self); // removed .into()
                                        let mut s = DefaultHasher::new();
                                        pk.hash(&mut s);
                                        let h = s.finish();
                                        let default_vec: Vec<usize>= Vec::new();
                                        let row_numbers : &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                                        let  n = row_offset;

                                        match field {
                                            #(#merge_code)*

                                            f @ _ => {
                                                return Err(
                                                     toql::error::ToqlError::SqlBuilderError(
                                                         toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(f.to_string()))
                                                     .into());
                                            }

                                        };

                                    }



                        }
                        Ok(())
                    }

                 }

                  impl<R,E> toql::tree::tree_merge::TreeMerge<R,E> for &mut #struct_ident
                 where  E: std::convert::From<toql::error::ToqlError>,
                 #struct_key_ident: toql::from_row::FromRow<R, E>,
                 #(#tree_merge_dispatch_bounds_ref)*
                {
                     #[allow(unused_mut)]
                     fn merge<'a, I>(  &mut self, mut descendents: &mut I, field: &str,
                                rows: &[R],row_offset: usize, index: &std::collections::HashMap<u64,Vec<usize>>, selection_stream: &toql::sql_builder::select_stream::SelectStream)
                        -> std::result::Result<(), E>
                         where I: Iterator<Item = toql::query::field_path::FieldPath<'a>>
                         {
                             <#struct_ident as toql::tree::tree_merge::TreeMerge<R,E>>::merge(self, descendents, field, rows, row_offset, index, selection_stream)
                         }
                }

        };

        log::debug!(
            "Source code for `{}`:\n{}",
            self.rust_struct.rust_struct_ident,
            mods.to_string()
        );
        tokens.extend(mods);
    }
}
