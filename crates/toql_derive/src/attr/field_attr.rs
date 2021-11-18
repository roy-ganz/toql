use super::literals::{parse_lit_str, set_unique_bool, set_unique_path_lit, set_unique_str_lit};
use crate::{
    error::DeriveError,
    parsed::field::{
        join_field::ColumnPair,
        merge_field::{MergeColumn, MergeMatch},
        FieldRoles,
    },
    result::Result,
};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Ident, Path};

#[derive(Debug)]
pub(crate) struct FieldAttr {
    pub(crate) name: syn::Ident,
    pub(crate) type_path: syn::Path,
    pub(crate) roles: FieldRoles,
    pub(crate) preselect: Option<bool>,
    pub(crate) skip_mut: Option<bool>,
    pub(crate) skip_wildcard: Option<bool>,
    pub(crate) skip: Option<bool>,
    pub(crate) join: Option<JoinAttr>,
    pub(crate) merge: Option<MergeAttr>,
    pub(crate) sql: Option<String>,
    pub(crate) column: Option<String>,
    pub(crate) handler: Option<Path>,
    pub(crate) aux_params: HashMap<String, String>,
    pub(crate) foreign_key: Option<bool>,
    pub(crate) key: Option<bool>,
}

impl FieldAttr {
    pub(crate) fn try_from(name: syn::Ident, type_path: syn::Path) -> Result<Self> {
        Ok(FieldAttr {
            name,
            type_path: type_path.clone(),
            roles: FieldRoles::default(),
            preselect: None,
            skip_mut: None,
            skip_wildcard: None,
            skip: None,
            join: None,
            merge: None,
            sql: None,
            column: None,
            handler: None,
            aux_params: HashMap::new(),
            foreign_key: None,
            key: None,
        })
    }
}

#[derive(Debug, Default)]
pub struct JoinAttr {
    pub on_sql: Option<String>,
    pub columns: Vec<ColumnPair>,
    pub partial_table: Option<bool>,
}

#[derive(Debug, Default)]
pub struct MergeAttr {
    pub columns: Vec<MergeMatch>,
    pub join_sql: Option<String>,
    pub on_sql: Option<String>,
}

impl FieldAttr {
    pub fn parse_field_meta(
        &mut self,
        nested_meta: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        fn available_err(ident: Ident) -> DeriveError {
            let available = [
                "preselect",
                "skip_mut",
                "skip_wildcard",
                "sql",
                "columns",
                "handler",
                "foreign_key",
                "key",
                "roles",
                "aux_params",
                "join",
                "merge",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            DeriveError::UnknownAttribute(ident, available)
        }

        for meta in nested_meta {
            // println!("META = {:?}", &meta);
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "preselect" => {
                            set_unique_bool(&mut self.preselect, ident, true)?;
                        }
                        "skip_mut" => {
                            set_unique_bool(&mut self.skip_mut, ident, true)?;
                        }
                        "skip" => {
                            set_unique_bool(&mut self.skip, ident, true)?;
                        }
                        "skip_wildcard" => {
                            set_unique_bool(&mut self.skip_wildcard, ident, true)?;
                        }
                        "foreign_key" => {
                            // Applies on field
                            set_unique_bool(&mut self.foreign_key, ident, true)?;
                        }
                        "key" => {
                            // For fields or joins
                            set_unique_bool(&mut self.key, ident, true)?;
                        }
                        "join" => {
                            // Shorthand for join
                            self.join = Some(JoinAttr::default());
                        }
                        "merge" => {
                            // Shorthand for merge
                            self.merge = Some(MergeAttr::default());
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            } else if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "sql" => {
                            set_unique_str_lit(&mut self.sql, ident, &lit)?;
                        }
                        "column" => {
                            set_unique_str_lit(&mut self.column, ident, &lit)?;
                        }
                        "handler" => {
                            // Applies on fields and predicates
                            set_unique_path_lit(&mut self.handler, ident, &lit)?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            } else if let syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList {
                path,
                nested,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "roles" => {
                            self.parse_roles_meta(ident, nested.into_iter())?;
                        }
                        "aux_param" => {
                            // field or join
                            Self::parse_aux_params_meta(
                                &mut self.aux_params,
                                ident,
                                nested.into_iter(),
                            )?;
                        }
                        "join" => {
                            Self::parse_join_meta(
                                &mut self.join.get_or_insert(JoinAttr::default()),
                                ident.clone(),
                                nested.into_iter(),
                            )?;
                        }
                        "merge" => {
                            Self::parse_merge_meta(
                                &mut self.merge.get_or_insert(MergeAttr::default()),
                                ident.clone(),
                                nested.into_iter(),
                            )?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }

        Ok(())
    }

    fn parse_join_meta(
        join_attr: &mut JoinAttr,
        ident: syn::Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        fn available_err(ident: syn::Ident) -> DeriveError {
            let available = ["on_sql", "columns", "partial_table"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            DeriveError::UnknownAttribute(ident, available)
        }
        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "partial_table" => {
                            set_unique_bool(&mut join_attr.partial_table, ident, true)?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(available_err(ident.clone()));
                }
            } else if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "on_sql" => {
                            set_unique_str_lit(&mut join_attr.on_sql, ident, &lit)?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    DeriveError::AttributeNameExpected(path.span());
                }
            } else if let syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList {
                path,
                nested,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "columns" => {
                            Self::parse_column_pair_meta(
                                &mut join_attr.columns,
                                ident,
                                nested.into_iter(),
                            )?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    DeriveError::AttributeNameExpected(path.span());
                }
            }
        }
        Ok(())
    }

    fn parse_merge_meta(
        merge_attr: &mut MergeAttr,
        _ident: syn::Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        fn available_err(ident: syn::Ident) -> DeriveError {
            let available = ["on_sql", "join_sql", "columns"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            DeriveError::UnknownAttribute(ident, available)
        }

        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "on_sql" => {
                            set_unique_str_lit(&mut merge_attr.on_sql, ident, &lit)?;
                        }
                        "join_sql" => {
                            set_unique_str_lit(&mut merge_attr.join_sql, ident, &lit)?;
                        }

                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            } else if let syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList {
                path,
                nested,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "columns" => {
                            Self::parse_merge_match_meta(
                                &mut merge_attr.columns,
                                ident,
                                nested.into_iter(),
                            )?;
                        }
                        _ => {
                            return Err(available_err(ident.clone()));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }
        Ok(())
    }
    fn parse_roles_meta(
        &mut self,
        _ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "load" => {
                            if self.roles.load.is_some() {
                                return Err(DeriveError::DuplicateAttribute(ident.clone()));
                            } else {
                                self.roles.load = Some(parse_lit_str(&lit)?);
                            }
                        }
                        "update" => {
                            if self.roles.update.is_some() {
                                return Err(DeriveError::DuplicateAttribute(ident.clone()));
                            } else {
                                self.roles.update = Some(parse_lit_str(&lit)?);
                            }
                        }
                        _ => {
                            let available =
                                ["load", "update"].iter().map(|s| s.to_string()).collect();
                            return Err(DeriveError::UnknownAttribute(ident.clone(), available));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }
        Ok(())
    }

    fn parse_aux_params_meta(
        aux_params: &mut HashMap<String, String>,
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        let mut name = None;
        let mut value = None;
        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "name" => {
                            set_unique_str_lit(&mut name, ident, &lit)?;
                        }
                        "value" => {
                            set_unique_str_lit(&mut value, ident, &lit)?;
                        }
                        _ => {
                            let available =
                                ["name", "value"].iter().map(|s| s.to_string()).collect();
                            return Err(DeriveError::UnknownAttribute(ident.clone(), available));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }
        let name =
            name.ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "name".to_string()))?;
        let value = value
            .ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "value".to_string()))?;
        aux_params.insert(name, value);

        Ok(())
    }
    fn parse_column_pair_meta(
        columns: &mut Vec<ColumnPair>,
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        let mut this = None;
        let mut other = None;
        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "self" => {
                            set_unique_str_lit(&mut this, ident, &lit)?;
                        }
                        "other" => {
                            set_unique_str_lit(&mut other, ident, &lit)?;
                        }
                        _ => {
                            let available =
                                ["self", "other"].iter().map(|s| s.to_string()).collect();
                            return Err(DeriveError::UnknownAttribute(ident.clone(), available));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }
        let this =
            this.ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "self".to_string()))?;
        let other = other
            .ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "other".to_string()))?;

        columns.push(ColumnPair { this, other });

        Ok(())
    }
    fn parse_merge_match_meta(
        columns: &mut Vec<MergeMatch>,
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        let mut this = None;
        let mut other = None;
        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "self" => {
                            set_unique_str_lit(&mut this, ident, &lit)?;
                        }
                        "other" => {
                            set_unique_str_lit(&mut other, ident, &lit)?;
                        }
                        _ => {
                            let available =
                                ["self", "other"].iter().map(|s| s.to_string()).collect();
                            return Err(DeriveError::UnknownAttribute(ident.clone(), available));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeNameExpected(path.span()));
                }
            }
        }
        let this =
            this.ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "self".to_string()))?;
        let other = other
            .ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "other".to_string()))?;

        columns.push(MergeMatch {
            this,
            other: if other.contains('.') {
                MergeColumn::Aliased(other)
            } else {
                MergeColumn::Unaliased(other)
            },
        });

        Ok(())
    }
}
