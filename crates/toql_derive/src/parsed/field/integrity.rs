use crate::{
    attr::field_attr::{FieldAttr, MergeAttr},
    error::DeriveError,
    result::Result,
};

use super::type_info::{TypeHint, TypeInfo};
use syn::spanned::Spanned;

pub(crate) fn check_skipped_attr_integrity(field_attr: &FieldAttr) -> Result<()> {
    if field_attr.skip_mut.unwrap_or_default()
        || field_attr.skip_wildcard.unwrap_or_default()
        || field_attr.join.is_some()
        || field_attr.merge.is_some()
        || field_attr.roles.load.is_some()
        || field_attr.roles.update.is_some()
        || field_attr.preselect.is_some()
        || field_attr.sql.is_some()
        || field_attr.column.is_some()
        || field_attr.handler.is_some()
        || !field_attr.aux_params.is_empty()
        || field_attr.foreign_key.unwrap_or_default()
        || field_attr.key.unwrap_or_default()
    {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "skipped fields do not allow other attributes.".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn check_regular_attr_integrity(
    field_attr: &FieldAttr,
    type_info: &TypeInfo,
) -> Result<()> {
    // Expect join
    if type_info.type_hint == TypeHint::Join || type_info.type_hint == TypeHint::Merge {
        return Err(DeriveError::InvalidType(field_attr.type_path.span()));
    }

    if field_attr.key.unwrap_or_default() && field_attr.skip_mut.unwrap_or_default() {
        return Err(DeriveError::Custom(field_attr.name.span(),
                "key must not be `skip_mut`. Use `#[toql(auto_key)]` on your struct, if your key is an auto value."
                    .to_string(),
            ));
    }
    if field_attr.key.unwrap_or_default() && field_attr.sql.is_some() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "key must not be an expression.".to_string(),
        ));
    }
    if field_attr.key.unwrap_or_default() && type_info.number_of_options > 0 {
        return Err(DeriveError::OptionalKey(field_attr.name.span()));
    }
    if field_attr.key.unwrap_or_default()
        && (field_attr.roles.load.is_some() || field_attr.roles.update.is_some())
    {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "Key must not be role restricted.".to_string(),
        ));
    }
    if field_attr.column.is_some() && field_attr.sql.is_some() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`column` and `sql` are not allowed together.".to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn check_join_attr_integrity(
    field_attr: &FieldAttr,
    type_info: &TypeInfo,
) -> Result<()> {
    // Expect join
    if !(type_info.type_hint == TypeHint::Join || type_info.type_hint == TypeHint::Other) {
        return Err(DeriveError::InvalidType(field_attr.type_path.span()));
    }
    if field_attr.sql.is_some() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`sql` not allowed for joins.".to_string(),
        ));
    }
    if field_attr.skip_wildcard.unwrap_or_default() && field_attr.preselect.unwrap_or_default() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`skip_wildcard` is not allowed together with `preselect`."
                .to_string(),
        ));
    }
    if field_attr.skip_wildcard.unwrap_or_default() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`skip_wildcard` not allowed for joins.".to_string(),
        ));
    }
    if field_attr.key.unwrap_or_default() && field_attr.skip_mut.unwrap_or_default() {
        return Err(DeriveError::Custom(field_attr.name.span(),
                "key must not be `skip_mut`. Use `#[toql(auto_key)]` on your struct, if your key is an auto value."
                    .to_string(),
            ));
    }

    if field_attr.key.unwrap_or_default() && type_info.number_of_options > 0 {
        return Err(DeriveError::OptionalKey(field_attr.name.span()));
    }

    if field_attr
        .join
        .as_ref()
        .map(|j| j.partial_table.unwrap_or_default())
        .unwrap_or_default()
        && field_attr.roles.update.is_some()
    {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "partial joins can't be update restricted, because no foreign key exists.".to_string(),
        ));
    }
    Ok(())
}
pub(crate) fn check_merge_attr_integrity(
    field_attr: &FieldAttr,
    merge_attr: &MergeAttr,
    type_hint: &TypeHint,
) -> Result<()> {
    // Expect merge
    if !(type_hint == &TypeHint::Merge || type_hint == &TypeHint::Other) {
        return Err(DeriveError::InvalidType(field_attr.type_path.span()));
    }
    if field_attr.sql.is_some() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`sql` not allowed for merged fields. ".to_string(),
        ));
    }
    if field_attr.preselect.unwrap_or_default() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`preselect` not allowed for merged fields.".to_string(),
        ));
    }
    if field_attr.key.unwrap_or_default() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`key` not allowed for merged fields.".to_string(),
        ));
    }
    if field_attr.handler.is_some() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`handler` not allowed for merged fields.".to_string(),
        ));
    }
    if field_attr.skip_wildcard.unwrap_or_default() {
        return Err(DeriveError::Custom(
            field_attr.name.span(),
            "`skip_wildcard` is not allowed for merged fields."
                .to_string(),
        ));
    }

    if let Some(j) = merge_attr.join_sql.as_ref() {
        // Search for .., ignore ...
        let mut n = 0;
        let found_self_alias = j.chars().any(|c| {
            if c == '.' {
                n += 1;
                false
            } else if n == 2 {
                true
            } else {
                n = 0;
                false
            }
        });
        if found_self_alias {
            return Err(DeriveError::Custom(field_attr.name.span(),
                "alias `..` not allowed for merged fields. Use `...` to refer to table of merged entities.".to_string(),
            ));
        }
    }

    Ok(())
}
