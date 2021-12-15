use super::{
    join_field::{JoinField, JoinSelection},
    merge_field::{MergeField, MergeSelection},
    regular_field::{RegularField, RegularSelection, SqlTarget},
    type_info::TypeInfo,
};
use crate::{
    attr::{
        field_attr::{FieldAttr, JoinAttr},
        struct_attr::StructAttr,
    },
    parsed::field::integrity::{
        check_join_attr_integrity, check_merge_attr_integrity, check_regular_attr_integrity,
        check_skipped_attr_integrity,
    },
};
use crate::{error::DeriveError, parsed::rename_case::RenameCase, result::Result};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
#[derive(Debug)]
pub(crate) enum FieldKind {
    Regular(RegularField),
    Join(JoinField),
    Merge(MergeField),
    Skipped,
}

impl FieldKind {
    #[cfg(test)]
    pub(crate) fn as_regular(&self) -> Option<&RegularField> {
        match self {
            FieldKind::Regular(s) => Some(s),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn as_join(&self) -> Option<&JoinField> {
        match self {
            FieldKind::Join(s) => Some(s),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn as_merge(&self) -> Option<&MergeField> {
        match self {
            FieldKind::Merge(m) => Some(m),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn is_skipped(self) -> bool {
        matches!(self, FieldKind::Skipped)
    }
}

pub(crate) fn build(
    struct_attr: &StructAttr,
    field_attr: &FieldAttr,
    type_info: &TypeInfo,
) -> Result<FieldKind> {
    use heck::{MixedCase, SnakeCase};

    if field_attr.skip.unwrap_or_default() {
        check_skipped_attr_integrity(field_attr)?;
        Ok(FieldKind::Skipped)
    } else if let Some(join_attr) = &field_attr.join {
        let selection = match (
            type_info.number_of_options,
            field_attr.preselect.unwrap_or_default(),
        ) {
            (2, false) => JoinSelection::SelectLeft,
            (1, false) => JoinSelection::SelectInner,
            (0, false) => JoinSelection::PreselectInner,
            (1, true) => JoinSelection::PreselectLeft,
            _ => return Err(DeriveError::InvalidType(field_attr.type_path.span())),
        };
        check_join_attr_integrity(field_attr, &type_info)?;

        if field_attr.key.unwrap_or_default() && type_info.number_of_options > 0 {
            return Err(DeriveError::OptionalKey(field_attr.type_path.span()));
        }

        let sql_join_table_name = struct_attr
            .tables
            .as_ref()
            .unwrap_or(&RenameCase::CamelCase)
            .rename_str(&type_info.base_name.to_string());
        let join_alias = field_attr
            .name
            .to_string()
            .trim_start_matches("r#")
            .to_mixed_case();

        let default_self_column_code = generate_default_self_column_code(field_attr);
        let columns_map_code =
            generate_columns_map_code(&struct_attr, field_attr, &join_attr, type_info);
        let translated_default_self_column_code = generate_translated_default_self_column_code();
        let translated_columns_map_code =
            generate_translated_columns_map_code(struct_attr, field_attr, &join_attr, type_info);

        Ok(FieldKind::Join(JoinField {
            sql_join_table_name,
            join_alias,
            default_self_column_code,
            columns_map_code,
            translated_default_self_column_code,
            translated_columns_map_code,
            on_sql: join_attr.on_sql.clone(),
            key: field_attr.key.unwrap_or_default(),
            aux_params: field_attr.aux_params.clone(),
            columns: join_attr.columns.clone(),
            partial_table: join_attr.partial_table.unwrap_or_default(),
            foreign_key: field_attr.foreign_key.unwrap_or_default(),
            selection,
            handler: field_attr.handler.clone(),
        }))
    } else if let Some(merge_attr) = &field_attr.merge {
        let selection = match type_info.number_of_options {
            1 => MergeSelection::Select,
            0 => MergeSelection::Preselect,
            _ => return Err(DeriveError::InvalidType(field_attr.type_path.span())),
        };
        check_merge_attr_integrity(field_attr, &merge_attr, &type_info)?;

        let sql_join_table_name = struct_attr
            .tables
            .as_ref()
            .unwrap_or(&RenameCase::CamelCase)
            .rename_str(&struct_attr.name.to_string());
        let join_alias = sql_join_table_name.to_snake_case();

        Ok(FieldKind::Merge(MergeField {
            sql_join_table_name,
            join_alias,
            columns: merge_attr.columns.clone(),
            join_sql: merge_attr.join_sql.clone(),
            on_sql: merge_attr.on_sql.clone(),
            selection,
        }))
    } else {
        let selection = match (
            type_info.number_of_options,
            field_attr.preselect.unwrap_or_default(),
        ) {
            (2, false) => RegularSelection::SelectNullable,
            (1, false) => RegularSelection::Select,
            (0, false) => RegularSelection::Preselect,
            (1, true) => RegularSelection::PreselectNullable,
            _ => return Err(DeriveError::InvalidType(field_attr.type_path.span())),
        };
        check_regular_attr_integrity(field_attr, &type_info)?;
        if field_attr.key.unwrap_or_default() && type_info.number_of_options > 0 {
            return Err(DeriveError::OptionalKey(field_attr.type_path.span()));
        }

        let default_inverse_column = if field_attr.sql.is_some() {
            None
        } else {
            let table_name = struct_attr.table.clone().unwrap_or_else(|| {
                struct_attr
                    .tables
                    .as_ref()
                    .unwrap_or(&RenameCase::CamelCase)
                    .rename_str(&struct_attr.name.to_string())
            });
            Some(
                struct_attr
                    .columns
                    .as_ref()
                    .unwrap_or(&RenameCase::SnakeCase)
                    .rename_str(&format!(
                        "{}_{}",
                        &table_name,
                        &field_attr.name.to_string().trim_start_matches("r#")
                    )),
            )
        };
        let sql_target = if let Some(sql) = &field_attr.sql {
            SqlTarget::Expression(sql.clone())
        } else {
            SqlTarget::Column(match &field_attr.column {
                Some(string) => string.to_owned(),
                None => struct_attr
                    .columns
                    .clone()
                    .unwrap_or(RenameCase::SnakeCase)
                    .rename_str(&field_attr.name.to_string().trim_start_matches("r#")),
            })
        };

        // Build regular field
        Ok(FieldKind::Regular(RegularField {
            sql_target,
            key: field_attr.key.unwrap_or_default(),
            handler: field_attr.handler.clone(),
            default_inverse_column,
            aux_params: field_attr.aux_params.clone(),
            foreign_key: field_attr.foreign_key.unwrap_or_default(),
            selection,
            skip_wildcard: field_attr.skip_wildcard.unwrap_or_default(),
        }))
    }
}

fn generate_translated_columns_map_code(
    struct_attr: &StructAttr,
    field_attr: &FieldAttr,
    join_attr: &JoinAttr,
    type_info: &TypeInfo,
) -> TokenStream {
    let safety_check_for_column_mapping =
        generate_safety_check_for_column_mapping(struct_attr, field_attr, join_attr, type_info);

    let translated_columns_translation = join_attr
        .columns
        .iter()
        .map(|column| {
            let tc = &column.this;
            let oc = &column.other;
            quote!(#oc => mapper.translate_aliased_column(sql_alias,#tc), )
        })
        .collect::<Vec<_>>();
    quote!( {

        #safety_check_for_column_mapping

        let self_column = match other_column.as_str(){
                #(#translated_columns_translation)*
                _ => default_self_column
        };
        self_column
    })
}

fn generate_translated_default_self_column_code() -> TokenStream {
    quote!( let default_self_column= mapper.translate_aliased_column(sql_alias, other_column);)
}

fn generate_safety_check_for_column_mapping(
    struct_attr: &StructAttr,
    field_attr: &FieldAttr,
    join_attrs: &JoinAttr,
    type_info: &TypeInfo,
) -> TokenStream {
    let other_columns: Vec<String> = join_attrs
        .columns
        .iter()
        .map(|column| String::from(column.other.as_str()))
        .collect::<Vec<_>>();

    let struct_name = &struct_attr.name.to_string();
    //let rust_type_ident = &field_attr.type_path;
    let rust_base_type_path = &type_info.base_type;
    let rust_field_name = &field_attr.name.to_string();
    if other_columns.is_empty() {
        quote!()
    } else {
        quote!(
            if cfg!(debug_assertions) {
                let valid_columns = <<#rust_base_type_path as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                let invalid_columns: Vec<String> = [ #(#other_columns),* ]
                    .iter()
                    .filter(|col| !valid_columns.iter().any ( |s| &s == col ) )
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>();

                if !invalid_columns.is_empty() {
                toql::tracing::warn!("On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`", #struct_name, #rust_field_name, invalid_columns.join(","),valid_columns.join(","));
                }
            }
        )
    }
}
fn generate_columns_map_code(
    struct_attr: &StructAttr,
    field_attr: &FieldAttr,
    join_attrs: &JoinAttr,
    type_info: &TypeInfo,
) -> TokenStream {
    let safety_check_for_column_mapping =
        generate_safety_check_for_column_mapping(struct_attr, field_attr, join_attrs, type_info);

    let columns_translation = join_attrs
        .columns
        .iter()
        .map(|column| {
            let tc = &column.this;
            let oc = &column.other;
            quote!(#oc => #tc, )
        })
        .collect::<Vec<_>>();

    quote!( {

        #safety_check_for_column_mapping

        let self_column = match other_column.as_str(){
                #(#columns_translation)*
                _ => &default_self_column
        };
        self_column
    })
}

fn generate_default_self_column_code(field_attr: &FieldAttr) -> TokenStream {
    let default_self_column_format = format!(
        "{}_{{}}",
        field_attr.name.to_string().trim_start_matches("r#")
    );
    quote!( let default_self_column= format!(#default_self_column_format, other_column);)
}
