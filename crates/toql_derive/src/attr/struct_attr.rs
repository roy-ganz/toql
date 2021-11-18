use super::literals::{
    parse_lit_str, set_unique_bool, set_unique_path_lit, set_unique_rename_case_lit,
    set_unique_str_lit, set_unique_usize_lit,
};
use crate::{
    error::DeriveError,
    parsed::{
        parsed_struct::StructRoles, predicate_arg::PredicateArg, rename_case::RenameCase,
        selection_arg::SelectionArg,
    },
    result::Result,
};
use heck::MixedCase;
use std::collections::HashMap;
use syn::{spanned::Spanned, Ident, Path};

#[derive(Debug)]
pub(crate) struct StructAttr {
    pub(crate) name: Ident,
    pub(crate) auto_key: Option<bool>,
    pub(crate) tables: Option<RenameCase>,
    pub(crate) table: Option<String>,
    pub(crate) columns: Option<RenameCase>,
    pub(crate) skip_mut: Option<bool>,
    pub(crate) predicates: HashMap<String, PredicateArg>,
    pub(crate) selections: HashMap<String, SelectionArg>,
    pub(crate) roles: StructRoles,
    pub(crate) handler: Option<Path>,
}

impl StructAttr {
    pub(crate) fn new(name: Ident) -> Self {
        StructAttr {
            name,
            auto_key: None,
            tables: None,
            table: None,
            columns: None,
            skip_mut: None,
            predicates: HashMap::new(),
            selections: HashMap::new(),
            roles: StructRoles::default(),
            handler: None,
        }
    }

    pub(crate) fn parse_meta(
        &mut self,
        nested_meta: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        fn available_err(ident: Ident) -> DeriveError {
            let available = [
                "auto_key",
                "table",
                "tables",
                "columns",
                "predicate",
                "selection",
                "handler",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            DeriveError::UnknownAttribute(ident, available)
        }

        for meta in nested_meta {
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "auto_key" => {
                            set_unique_bool(&mut self.auto_key, ident, true)?;
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
                        "table" => {
                            set_unique_str_lit(&mut self.table, ident, &lit)?;
                        }
                        "tables" => {
                            set_unique_rename_case_lit(&mut self.tables, ident, &lit)?;
                        }
                        "columns" => {
                            set_unique_rename_case_lit(&mut self.columns, ident, &lit)?;
                        }
                        "handler" => {
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
                        "predicate" => {
                            self.parse_predicate_meta(ident, nested.into_iter())?;
                        }
                        "selection" => {
                            self.parse_selection_meta(ident, nested.into_iter())?;
                        }
                        "roles" => {
                            self.parse_roles_meta(ident, nested.into_iter())?;
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

    pub(crate) fn parse_roles_meta(
        &mut self,
        _ident: &Ident,
        nested_meta: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        for meta in nested_meta {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = meta
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "insert" => {
                            set_unique_str_lit(&mut self.roles.insert, ident, &lit)?;
                        }
                        "update" => {
                            set_unique_str_lit(&mut self.roles.update, ident, &lit)?;
                        }
                        "delete" => {
                            set_unique_str_lit(&mut self.roles.delete, ident, &lit)?;
                        }
                        "load" => {
                            set_unique_str_lit(&mut self.roles.load, ident, &lit)?;
                        }
                        _ => {
                            let available = ["insert", "update", "load", "delete"]
                                .iter()
                                .map(|s| s.to_string())
                                .collect();
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
    pub(crate) fn parse_selection_meta(
        &mut self,
        ident: &Ident,
        nested_meta: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        let mut name = None;
        let mut fields = None;
        for meta in nested_meta {
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
                        "fields" => {
                            set_unique_str_lit(&mut fields, ident, &lit)?;
                        }
                        _ => {
                            let available =
                                ["name", "fields"].iter().map(|s| s.to_string()).collect();
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
        let fields = fields
            .ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "fields".to_string()))?;

        if name.len() <= 3 && name != "std" && name != "cnt" {
            return Err(DeriveError::InvalidAttribute(
                ident.span(),
                "Selection name must be `std`, `cnt` or longer than 3 characters".to_string(),
            ));
        }

        self.selections
            .insert(name.to_mixed_case(), SelectionArg { fields });

        Ok(())
    }

    pub(crate) fn parse_predicate_meta(
        &mut self,
        ident: &Ident,
        nested_meta: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        fn available_err(ident: Ident) -> DeriveError {
            let available = ["name", "sql", "count_filter", "handler", "on_aux_param"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            DeriveError::UnknownAttribute(ident, available)
        }

        let mut name = None; // Required
        let mut sql = None; // Required
        let mut handler = None; // Default to None
        let mut count_filter = None; // Defaults to false
        let mut on_aux_params = std::collections::HashMap::new(); // Defaults to empty

        for n in nested_meta {
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = n {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "count_filter" => {
                            set_unique_bool(&mut count_filter, ident, true)?;
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
            })) = n
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "name" => {
                            set_unique_str_lit(&mut name, ident, &lit)?;
                        }
                        "sql" => {
                            set_unique_str_lit(&mut sql, ident, &lit)?;
                        }
                        "handler" => {
                            set_unique_path_lit(&mut handler, ident, &lit)?;
                        }
                        "on_aux_param" => {
                            let str = parse_lit_str(&lit)?;
                            if on_aux_params.contains_key(&str) {
                                return Err(DeriveError::DuplicateAttribute(ident.clone()));
                            } else {
                                on_aux_params.insert(str, 0);
                            }
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
            })) = n
            {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "on_aux_param" => {
                            let mut name = None;
                            let mut index = None;
                            for n in nested {
                                if let syn::NestedMeta::Meta(syn::Meta::NameValue(
                                    syn::MetaNameValue { path, lit, .. },
                                )) = n
                                {
                                    if let Some(ident) = path.get_ident() {
                                        match ident.to_string().as_str() {
                                            "name" => {
                                                set_unique_str_lit(&mut name, ident, &lit)?;
                                            }
                                            "index" => {
                                                set_unique_usize_lit(&mut index, ident, &lit)?;
                                            }
                                            _ => {
                                                let available = ["name", "index"]
                                                    .iter()
                                                    .map(|s| s.to_string())
                                                    .collect();
                                                return Err(DeriveError::UnknownAttribute(
                                                    ident.clone(),
                                                    available,
                                                ));
                                            }
                                        }
                                    } else {
                                        return Err(DeriveError::AttributeNameExpected(
                                            path.span(),
                                        ));
                                    }
                                }
                            }
                            let name = name.ok_or_else(|| {
                                DeriveError::AttributeMissing(ident.clone(), "name".to_string())
                            })?;
                            on_aux_params.insert(name, index.unwrap_or(0));
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

        let name =
            name.ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "name".to_string()))?;
        let sql =
            sql.ok_or_else(|| DeriveError::AttributeMissing(ident.clone(), "sql".to_string()))?;
        let predicate_arg = PredicateArg {
            sql,
            handler, //: handler.map(|h| Ident::new(&h, Span::call_site())),
            count_filter: count_filter.unwrap_or(false),
            on_aux_params,
        };

        self.predicates.insert(name.to_mixed_case(), predicate_arg);

        Ok(())
    }
}
