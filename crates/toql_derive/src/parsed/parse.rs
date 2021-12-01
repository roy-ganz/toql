use super::{
    field::field_kind::FieldKind, field::Field, parsed_struct::ParsedStruct,
    rename_case::RenameCase,
};
use crate::attr::{field_attr::FieldAttr, struct_attr::StructAttr};
use crate::error::DeriveError;
use syn::parse::{Parse, ParseStream};

impl Parse for ParsedStruct {
    fn parse(input: ParseStream) -> syn::Result<ParsedStruct> {
        // Parse struct with attributes
        let derive_input: syn::DeriveInput = input.parse()?;

        let mut struct_attr = StructAttr::new(derive_input.ident.clone());
        let mut parsed_fields = Vec::new();

        // Parse #[toql(..)], if available
        for attr in derive_input.attrs.iter() {
            let meta = attr.parse_meta()?;

            if let syn::Meta::List(syn::MetaList { path, nested, .. }) = meta {
                if let Some(ident) = path.get_ident() {
                    if ident == &"toql" {
                        struct_attr
                            .parse_meta(nested.into_iter())
                            .map_err::<syn::Error, _>(|e| e.into())?;
                    }
                }
            }
        }

        // Parse struct fields with attributes
        if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &derive_input.data {
            parsed_fields = parse_struct_field(&struct_attr, fields.iter())
                .map_err::<syn::Error, _>(|e| e.into())?;
        }

        check_key_integrity(&derive_input.ident, parsed_fields.iter())?;

        // Table name is either user defined or calculated from struct name and renaming scheme
        let tables = struct_attr.tables.unwrap_or(RenameCase::CamelCase);
        let table = struct_attr
            .table
            .unwrap_or_else(|| tables.rename_str(&derive_input.ident.to_string()));

        Ok(ParsedStruct {
            vis: derive_input.vis,
            struct_name: derive_input.ident,
            tables,
            table,
            columns: struct_attr.columns.unwrap_or(RenameCase::CamelCase),
            skip_mut: false,
            auto_key: struct_attr.auto_key.unwrap_or(false),
            predicates: struct_attr.predicates,
            selections: struct_attr.selections,
            fields: parsed_fields,
            roles: struct_attr.roles,
            field_handler: struct_attr.handler,
        })
    }
}

pub(crate) fn parse_struct_field<'a>(
    struct_attr: &StructAttr,
    fields: impl Iterator<Item = &'a syn::Field>,
) -> syn::Result<Vec<Field>> {
    use syn::spanned::Spanned;

    let mut parsed_fields = Vec::new();
    for field in fields {
        if let Some(ident) = &field.ident {
            if let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty {
                             
                let mut field_attr = FieldAttr::new(ident.clone(), path.clone());

                // Parse fields attributes
                for attr in &field.attrs {
                    let meta = attr.parse_meta()?;
                    if let syn::Meta::List(syn::MetaList { path, nested, .. }) = meta {
                        if let Some(ident) = path.get_ident() {
                            if ident == &"toql" {
                                field_attr
                                    .parse_field_meta(nested.into_iter())
                                    .map_err::<syn::Error, _>(|e| e.into())?;
                            }
                        }
                    }
                }
                parsed_fields.push(
                    Field::try_from(struct_attr, field_attr)
                        .map_err::<syn::Error, _>(|e| e.into())?,
                );
            }
        }
    }

    Ok(parsed_fields)
}

/// Check integrity
/// - (Composite) key must be present
/// - Key must be first field(s)

pub(crate) fn check_key_integrity<'a>(
    struct_name: &syn::Ident,
    fields: impl Iterator<Item = &'a Field>,
) -> syn::Result<()> {
    fn is_key(field: &Field) -> bool {
        match &field.kind {
            FieldKind::Regular(regular_kind) => regular_kind.key,
            FieldKind::Join(join_kind) => join_kind.key,
            FieldKind::Merge(_) => false,
            FieldKind::Skipped => false,
        }
    }

    let mut start_with_key = false;

    let key_trailing_field = fields
        .skip_while(|f| {
            if is_key(f) {
                start_with_key = true;
                true
            } else {
                false
            }
        })
        .find(|f| is_key(f));

    if let Some(trailing_key_field) = key_trailing_field {
        return Err(DeriveError::KeyTrailing(trailing_key_field.field_name.span().clone()).into());
    }
    if !start_with_key {
        return Err(DeriveError::KeyMissing(struct_name.span().clone()).into());
    }

    Ok(())
}
