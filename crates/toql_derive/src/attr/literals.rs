use crate::result::Result;
use crate::{error::DeriveError, parsed::rename_case::RenameCase};
use syn::{Ident, Lit, Path};

pub(crate) fn parse_lit_str(lit: &Lit) -> Result<String> {
    if let Lit::Str(lit_str) = lit {
        Ok(lit_str.value())
    } else {
        Err(DeriveError::AttributeValueInvalid(lit.span()))
    }
}
pub(crate) fn parse_lit_str_as_path(lit: &Lit) -> Result<Path> {
    if let Lit::Str(lit_str) = lit {
        lit_str
            .parse()
            .map_err(|_| DeriveError::AttributeValueInvalid(lit.span()))
    } else {
        Err(DeriveError::AttributeValueInvalid(lit.span()))
    }
}
pub(crate) fn parse_lit_u64(lit: &Lit) -> Result<u64> {
    if let Lit::Int(lit_int) = lit {
        lit_int
            .base10_parse::<u64>()
            .map_err(|_| DeriveError::AttributeValueInvalid(lit.span()))
    } else {
        Err(DeriveError::AttributeValueInvalid(lit.span()))
    }
}
pub(crate) fn set_unique_bool(value: &mut Option<bool>, ident: &Ident, b: bool) -> Result<()> {
    if value.is_some() {
        return Err(DeriveError::AttributeDuplicate(ident.span()));
    } else {
        *value = Some(b);
    }
    Ok(())
}
pub(crate) fn set_unique_str_lit(
    value: &mut Option<String>,
    ident: &Ident,
    lit: &Lit,
) -> Result<()> {
    if value.is_some() {
        return Err(DeriveError::AttributeDuplicate(ident.span()));
    } else {
        *value = Some(parse_lit_str(lit)?);
    }
    Ok(())
}
pub(crate) fn set_unique_path_lit(
    value: &mut Option<Path>,
    ident: &Ident,
    lit: &Lit,
) -> Result<()> {
    if value.is_some() {
        return Err(DeriveError::AttributeDuplicate(ident.span()));
    } else {
        *value = Some(parse_lit_str_as_path(lit)?);
    }
    Ok(())
}
pub(crate) fn set_unique_usize_lit(
    value: &mut Option<usize>,
    ident: &Ident,
    lit: &Lit,
) -> Result<()> {
    if value.is_some() {
        return Err(DeriveError::AttributeDuplicate(ident.span()));
    } else {
        *value = Some(parse_lit_u64(lit)? as usize);
    }
    Ok(())
}
pub(crate) fn set_unique_rename_case_lit(
    value: &mut Option<RenameCase>,
    ident: &Ident,
    lit: &Lit,
) -> Result<()> {
    if let Lit::Str(lit_str) = lit {
        let lit = lit_str.value();
        match lit.parse() {
            Ok(c) => {
                if value.is_some() {
                    return Err(DeriveError::AttributeDuplicate(ident.span()));
                } else {
                    *value = Some(c);
                }
            }
            Err(_) => {
                let expected: Vec<String> =
                    RenameCase::VARIANTS.iter().map(|s| s.to_string()).collect();
                return Err(DeriveError::AttributeValueUnknown(
                    lit_str.span(),
                    expected.join(", "),
                )
                .into());
            }
        }
    } else {
        return Err(DeriveError::AttributeValueInvalid(lit.span()));
    }
    Ok(())
}
