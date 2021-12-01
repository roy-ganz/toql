use super::literals::{
    parse_lit_str, set_unique_bool, set_unique_path_lit, set_unique_rename_case_lit,
    set_unique_str_lit, set_unique_usize_lit,
};
use crate::{
    error::{attribute_err, DeriveError},
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
        const KEYWORDS: &[&str] = &[
            "auto_key",
            "table",
            "tables",
            "columns",
            "predicate",
            "selection",
            "handler",
        ];

        for meta in nested_meta {
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "auto_key" => {
                            set_unique_bool(&mut self.auto_key, ident, true)?;
                        }
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
        const KEYWORDS: &[&str] = &["insert", "update", "delete", "load"];
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
        const KEYWORDS: &[&str] = &["name", "fields"];
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
                }
            }
        }
        let name =
            name.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "name".to_string()))?;
        let fields = fields
            .ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "fields".to_string()))?;

        if name.len() <= 3 && name != "std" && name != "cnt" {
            return Err(DeriveError::Custom(
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
        const KEYWORDS: &[&str] = &["name", "sql", "count_filter", "handler", "on_aux_param"];

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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
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
                            Self::parse_on_aux_params_meta(
                                &ident,
                                nested.into_iter(),
                                &mut on_aux_params,
                            )?;
                        }
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
                }
            }
        }

        let name =
            name.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "name".to_string()))?;
        let sql =
            sql.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "sql".to_string()))?;
        let predicate_arg = PredicateArg {
            sql,
            handler, //: handler.map(|h| Ident::new(&h, Span::call_site())),
            count_filter: count_filter.unwrap_or(false),
            on_aux_params,
        };

        self.predicates.insert(name.to_mixed_case(), predicate_arg);

        Ok(())
    }

    fn parse_on_aux_params_meta(
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        on_aux_params: &mut HashMap<String, usize>,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["name", "index"];
        let mut name = None;
        let mut index = None;
        for n in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
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
                        "index" => {
                            set_unique_usize_lit(&mut index, ident, &lit)?;
                        }
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
                }
            }
        }
        let name =
            name.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "name".to_string()))?;
        let index = index
            .ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "index".to_string()))?;

        if on_aux_params.contains_key(&name) {
            return Err(DeriveError::AttributeDuplicate(ident.span()));
        }

        on_aux_params.insert(name, index);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::StructAttr;
    use crate::error::DeriveError;
    use proc_macro2::Span;
    use std::iter::once;
    use syn::{Ident, NestedMeta, Path};

    fn create_struct() -> StructAttr {
        StructAttr::new(Ident::new("User", Span::call_site()))
    }

    #[test]
    fn parse_auto_key() {
        // Succesful case
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>("auto_key").unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        assert_eq!(struct_attr.auto_key, Some(true));
    }
    #[test]
    fn parse_handler() {
        // Succesful case
        let input = r#"handler="abc""#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        assert_eq!(
            struct_attr.handler,
            Some(Path::from(Ident::new("abc", Span::call_site())))
        );

        // Bad argument type
        let input = r#"handler=abc"#;
        assert!(syn::parse_str::<NestedMeta>(input).is_err());

        // Missing argument
        let input = r#"handler"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );

        // Unexpected list
        let input = r#"handler()"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_table() {
        // Succesful case
        let input = r#"table="ABC""#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        assert_eq!(struct_attr.table, Some("ABC".to_string()));

        // Bad argument type
        let input = r#"table=ABC"#;
        assert!(syn::parse_str::<NestedMeta>(input).is_err());

        // Missing argument
        let input = r#"table"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );

        // Unexpected list
        let input = r#"table()"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );
    }

    #[test]
    fn parse_tables() {
        use crate::parsed::rename_case::RenameCase;

        // Succesful cases
        for case in &RenameCase::VARIANTS {
            let mut struct_attr = create_struct();
            let meta = syn::parse_str::<NestedMeta>(&format!("tables=\"{}\"", case)).unwrap();
            struct_attr.parse_meta(once(meta)).unwrap();
            assert_eq!(
                struct_attr.tables,
                Some(case.parse::<RenameCase>().unwrap())
            );
        }

        // Bad argument type
        let input = r#"tables=CamelCase"#;
        assert!(syn::parse_str::<NestedMeta>(input).is_err());

        // Invalid case
        let input = r#"tables="ABC""#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeValueUnknown(Span::call_site(), RenameCase::VARIANTS.join(", "))
                .to_string()
        );

        // Missing argument
        let input = r#"tables"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );

        // Unexpected list
        let input = r#"table()"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_columns() {
        use crate::parsed::rename_case::RenameCase;

        // Succesful cases
        for case in &RenameCase::VARIANTS {
            let mut struct_attr = create_struct();
            let meta = syn::parse_str::<NestedMeta>(&format!("columns=\"{}\"", case)).unwrap();
            struct_attr.parse_meta(once(meta)).unwrap();
            assert_eq!(
                struct_attr.columns,
                Some(case.parse::<RenameCase>().unwrap())
            );
        }

        // Bad argument type
        let input = r#"columns=CamelCase"#;
        assert!(syn::parse_str::<NestedMeta>(input).is_err());

        // Missing argument
        let input = r#"columns"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );

        // Unexpected list
        let input = r#"columns()"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_selection() {
        // Succesful case
        let input = r#"selection(name="sela", fields="b")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        let a = struct_attr.selections.get("sela").unwrap();
        assert_eq!(a.fields, "b");

        // Name too short case
        let input = r#"selection(name="sel", fields="b")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        assert!(struct_attr.parse_meta(once(meta)).is_err());

        // Missing name
        let input = r#"selection(fields="b1")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "name".to_string()).to_string()
        );

        // Missing fields
        let input = r#"selection(name="sela")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "fields".to_string()).to_string()
        );
    }
    #[test]
    fn parse_predicate() {
        // Succesful case
        let input = r#"predicate(name="a", sql="s", handler="h", count_filter, on_aux_param(name="p", index = 0))"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        let a = struct_attr.predicates.get("a").unwrap();
        assert_eq!(a.sql, "s".to_string());
        assert_eq!(
            a.handler,
            Some(Path::from(Ident::new("h", Span::call_site())))
        );
        assert_eq!(a.count_filter, true);
        assert_eq!(a.on_aux_params.len(), 1);

        // Missing name
        let input = r#"predicate(sql="s1")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "name".to_string()).to_string()
        );

        // Missing sql
        let input = r#"predicate(name="sela")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "sql".to_string()).to_string()
        );
        // Duplicate aux param
        let input = r#"predicate(name="sela", sql="s",on_aux_param(name="b", index = 0), on_aux_param(name="b", index = 1))"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_roles() {
        // Succesful case
        let input = r#"roles(load="l", update="u", insert="i", delete="d")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        assert_eq!(struct_attr.roles.load, Some("l".to_string()));
        assert_eq!(struct_attr.roles.update, Some("u".to_string()));
        assert_eq!(struct_attr.roles.insert, Some("i".to_string()));
        assert_eq!(struct_attr.roles.delete, Some("d".to_string()));

        // Tolerate missing value
        let input = r#"roles()"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        struct_attr.parse_meta(once(meta)).unwrap();
        assert!(struct_attr.roles.load.is_none());
        assert!(struct_attr.roles.update.is_none());

        // Duplicate roles
        let input = r#"roles(load="l", load="u")"#;
        let mut struct_attr = create_struct();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = struct_attr.parse_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
    }
}
