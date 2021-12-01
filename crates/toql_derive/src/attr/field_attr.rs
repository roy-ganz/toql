use super::literals::{parse_lit_str, set_unique_bool, set_unique_path_lit, set_unique_str_lit};
use crate::{
    error::{attribute_err, DeriveError},
    parsed::field::{
        join_field::ColumnPair,
        merge_field::{MergeColumn, MergeMatch},
        FieldRoles,
    },
    result::Result,
};
use std::collections::HashMap;
use syn::{spanned::Spanned, Ident, Path};

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
    pub(crate) fn new(name: syn::Ident, type_path: syn::Path) -> Self {
        FieldAttr {
            name,
            type_path,
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
        }
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
        const KEYWORDS: &[&str] = &[
            "preselect",
            "skip_mut",
            "skip_wildcard",
            "sql",
            "column",
            "handler",
            "foreign_key",
            "key",
            "roles",
            "aux_params",
            "join",
            "merge",
        ];

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
                        "roles" => {
                            self.parse_roles_meta(nested.into_iter())?;
                        }
                        "aux_param" => {
                            // field or join
                            Self::parse_aux_params_meta(
                                ident,
                                nested.into_iter(),
                                &mut self.aux_params,
                            )?;
                        }
                        "join" => {
                            Self::parse_join_meta(
                                nested.into_iter(),
                                &mut self.join.get_or_insert(JoinAttr::default()),
                            )?;
                        }
                        "merge" => {
                            Self::parse_merge_meta(
                                nested.into_iter(),
                                &mut self.merge.get_or_insert(MergeAttr::default()),
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

        Ok(())
    }

    fn parse_join_meta(
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        join_attr: &mut JoinAttr,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["on_sql", "columns", "partial_table"];

        for meta in nested_metas {
            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "partial_table" => {
                            set_unique_bool(&mut join_attr.partial_table, ident, true)?;
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
                        "on_sql" => {
                            set_unique_str_lit(&mut join_attr.on_sql, ident, &lit)?;
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
                        "columns" => {
                            Self::parse_column_pair_meta(
                                ident,
                                nested.into_iter(),
                                &mut join_attr.columns,
                            )?;
                        }
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    DeriveError::AttributeExpected(path.span());
                }
            }
        }
        Ok(())
    }

    fn parse_merge_meta(
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        merge_attr: &mut MergeAttr,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["on_sql", "columns", "join_sql"];

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

                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    DeriveError::AttributeExpected(path.span());
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
                                ident,
                                nested.into_iter(),
                                &mut merge_attr.columns,
                            )?;
                        }
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    DeriveError::AttributeExpected(path.span());
                }
            }
        }
        Ok(())
    }
    fn parse_roles_meta(
        &mut self,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["load", "update"];
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
                                return Err(DeriveError::AttributeDuplicate(ident.span()));
                            } else {
                                self.roles.load = Some(parse_lit_str(&lit)?);
                            }
                        }
                        "update" => {
                            if self.roles.update.is_some() {
                                return Err(DeriveError::AttributeDuplicate(ident.span()));
                            } else {
                                self.roles.update = Some(parse_lit_str(&lit)?);
                            }
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

    fn parse_aux_params_meta(
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        aux_params: &mut HashMap<String, String>,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["name", "value"];
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
        let value = value
            .ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "value".to_string()))?;
        aux_params.insert(name, value);

        Ok(())
    }
    fn parse_column_pair_meta(
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        columns: &mut Vec<ColumnPair>,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["self", "other"];

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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
                }
            }
        }
        let this =
            this.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "self".to_string()))?;
        let other = other
            .ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "other".to_string()))?;

        columns.push(ColumnPair { this, other });

        Ok(())
    }
    fn parse_merge_match_meta(
        ident: &Ident,
        nested_metas: impl Iterator<Item = syn::NestedMeta>,
        columns: &mut Vec<MergeMatch>,
    ) -> Result<()> {
        const KEYWORDS: &[&str] = &["self", "other"];
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
                        keyword => {
                            return Err(attribute_err(ident.span(), keyword, KEYWORDS));
                        }
                    }
                } else {
                    return Err(DeriveError::AttributeExpected(path.span()));
                }
            }
        }
        let this =
            this.ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "self".to_string()))?;
        let other = other
            .ok_or_else(|| DeriveError::AttributeRequired(ident.span(), "other".to_string()))?;

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

#[cfg(test)]
mod test {
    use super::FieldAttr;
    use crate::error::DeriveError;
    use proc_macro2::Span;
    use std::iter::once;
    use syn::{Ident, NestedMeta, Path};

    fn create_field() -> FieldAttr {
        FieldAttr::new(
            Ident::new("field", Span::call_site()),
            Ident::new("u64", Span::call_site()).into(),
        )
    }

    #[test]
    fn parse_invalid_flags() {
        let keywords = &[
            "preselect",
            "skip_mut",
            "skip_wildcard",
            "foreign_key",
            "key",
        ];

        for keyword in keywords {
            let mut field_attr = create_field();

            // Unexpected argument type
            let meta = syn::parse_str::<NestedMeta>(&format!("{}=true", keyword)).unwrap();
            let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
            assert_eq!(
                err.to_string(),
                DeriveError::AttributeInvalid(Span::call_site()).to_string()
            );

            // Unexpected list
            let mut field_attr = create_field();
            let meta = syn::parse_str::<NestedMeta>(&format!("{}()", keyword)).unwrap();
            let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
            assert_eq!(
                err.to_string(),
                DeriveError::AttributeInvalid(Span::call_site()).to_string()
            );
        }
    }

    #[test]
    fn parse_preselect() {
        // Succesful case
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>("preselect").unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.preselect.is_some());
    }
    #[test]
    fn parse_skip_mut() {
        // Succesful case
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>("skip_mut").unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.skip_mut.is_some());
    }
    #[test]
    fn parse_skip_wildcard() {
        // Succesful case
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>("skip_wildcard").unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.skip_wildcard.is_some());
    }
    #[test]
    fn parse_foreign_key() {
        // Succesful case
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>("foreign_key").unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.foreign_key.is_some());
    }
    #[test]
    fn parse_key() {
        // Succesful case
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>("key").unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.key.is_some());
    }

    #[test]
    fn parse_sql() {
        // Succesful case
        let input = r#"sql="ABC""#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert_eq!(field_attr.sql, Some("ABC".to_string()));

        // Bad argument type
        let input = r#"sql=ABC"#;
        assert!(syn::parse_str::<NestedMeta>(input).is_err());

        // Missing argument
        let input = r#"sql"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );

        // Unexpected list
        let input = r#"sql()"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeInvalid(Span::call_site()).to_string()
        );
    }

    #[test]
    fn parse_invalid_arg_with_value() {
        let keywords = ["sql", "column"];

        for keyword in &keywords {
            // Bad argument type
            assert!(syn::parse_str::<NestedMeta>(&format!("{}=ABC", keyword)).is_err());

            // Missing argument
            let mut field_attr = create_field();
            let meta = syn::parse_str::<NestedMeta>(keyword).unwrap();
            let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
            assert_eq!(
                err.to_string(),
                DeriveError::AttributeInvalid(Span::call_site()).to_string()
            );

            // Unexpected list
            let mut field_attr = create_field();
            let meta = syn::parse_str::<NestedMeta>(&format!("{}()", keyword)).unwrap();
            let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
            assert_eq!(
                err.to_string(),
                DeriveError::AttributeInvalid(Span::call_site()).to_string()
            );
        }
    }

    #[test]
    fn parse_column() {
        // Succesful case
        let input = r#"column="ABC""#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert_eq!(field_attr.column, Some("ABC".to_string()));
    }

    #[test]
    fn parse_handler() {
        // Succesful case
        let input = r#"handler="abc""#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert_eq!(
            field_attr.handler,
            Some(Path::from(Ident::new("abc", Span::call_site())))
        );
    }
    #[test]
    fn parse_aux_params() {
        // Succesful case
        let input = r#"aux_param(name="a", value="b")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        let a = field_attr.aux_params.get("a").unwrap();
        assert_eq!(a, "b");

        // Missing name
        let input = r#"aux_param(value="b1")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "name".to_string()).to_string()
        );

        // Missing value
        let input = r#"aux_param(name="a1")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeRequired(Span::call_site(), "value".to_string()).to_string()
        );
    }
    #[test]
    fn parse_roles() {
        // Succesful case
        let input = r#"roles(load="l", update="u")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert_eq!(field_attr.roles.load, Some("l".to_string()));
        assert_eq!(field_attr.roles.update, Some("u".to_string()));

        // Tolerate missing value
        let input = r#"roles()"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.roles.load.is_none());
        assert!(field_attr.roles.update.is_none());

        // Duplicate roles
        let input = r#"roles(load="l", load="u")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_join() {
        // Succesful case
        let input = r#"join"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.join.is_some());

        // Succesful case
        let input = r#"join(columns(self="s1", other="o2"), columns(self="a2", other="o2"), on_sql="pred1", partial_table)"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        let join = field_attr.join.unwrap();

        assert_eq!(join.columns.len(), 2);
        assert_eq!(join.on_sql, Some("pred1".to_string()));
        assert_eq!(join.partial_table, Some(true));

        // Duplicate on_sql
        let input = r#"join(on_sql="pred1", on_sql="pred2")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
    }
    #[test]
    fn parse_merge() {
        use crate::parsed::field::merge_field::MergeColumn;

        // Succesful case
        let input = r#"merge"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        assert!(field_attr.merge.is_some());

        // Succesful case
        let input = r#"merge(columns(self="s1", other="o2"), columns(self="a2", other="o2"), on_sql="pred1", join_sql="join1")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        let merge = field_attr.merge.unwrap();

        assert_eq!(merge.columns.len(), 2);
        assert_eq!(merge.on_sql, Some("pred1".to_string()));
        assert_eq!(merge.join_sql, Some("join1".to_string()));

        // Aliased merge column
        let input = r#"merge(columns(self="s1", other="a.o2"))"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        field_attr.parse_field_meta(once(meta)).unwrap();
        let merge = field_attr.merge.unwrap();
        let column = merge.columns.get(0).unwrap();

        assert_eq!(column.this, "s1".to_string());
        assert_eq!(column.other, MergeColumn::Aliased("a.o2".to_string()));

        // Duplicate on_sql
        let input = r#"merge(on_sql="pred1", on_sql="pred2")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
        // Duplicate join_sql
        let input = r#"merge(join_sql="pred1", join_sql="pred2")"#;
        let mut field_attr = create_field();
        let meta = syn::parse_str::<NestedMeta>(input).unwrap();
        let err = field_attr.parse_field_meta(once(meta)).err().unwrap();
        assert_eq!(
            err.to_string(),
            DeriveError::AttributeDuplicate(Span::call_site()).to_string()
        );
    }
}
