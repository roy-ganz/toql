use crate::error::DeriveError;
use crate::result::Result;
use syn::{spanned::Spanned, Ident, Path};

#[derive(Debug, PartialEq)]
pub(crate) enum TypeHint {
    Join,
    Merge,
    Other,
}
pub(crate) struct TypeInfo {
    pub number_of_options: u8,
    pub type_hint: TypeHint,
    pub base_name: syn::Ident, // Other
    pub base_type: syn::Path,  // other::Other
}

pub(crate) fn get_type_info(type_path: &syn::Path) -> Result<TypeInfo> {
    let mut number_of_options = 0;
    let mut type_hint = TypeHint::Other;

    // Only look at last segment
    // my_crate::my_mod::my_type <- my_type is last segment
    // println!("TY = {:?}", &type_path);
    let last_segment = type_path.segments.last();

    if let Some(seg) = last_segment {
        match seg.ident.to_string().as_str() {
            "Option" => number_of_options += 1,
            "Vec" => type_hint = TypeHint::Merge,
            "Join" => type_hint = TypeHint::Join,
            _ => {}
        }
        // Look at generic types
        // T
        // Option<Option<T>>

        let mut base_name = seg.ident.clone();
        let mut base_type = type_path.clone();

        match &seg.arguments {
            syn::PathArguments::None => {
                return Ok(TypeInfo {
                    number_of_options,
                    type_hint,
                    base_name: seg.ident.clone(),
                    base_type: type_path.clone(),
                })
            }
            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) => {
                eval_arguments(
                    args.into_iter(),
                    &mut number_of_options,
                    &mut type_hint,
                    &mut base_name,
                    &mut base_type,
                )?;
                return Ok(TypeInfo {
                    number_of_options,
                    type_hint,
                    base_name,
                    base_type,
                });
            }
            _ => return Err(DeriveError::InvalidType(seg.span())),
        }
    }

    Err(DeriveError::InvalidType(type_path.span()))
}

fn eval_arguments<'a>(
    args: impl Iterator<Item = &'a syn::GenericArgument>,
    number_of_options: &mut u8,
    type_hint: &mut TypeHint,
    base_name: &mut Ident,
    base_type: &mut Path,
) -> Result<()> {
    for a in args {
        // println!("ARG = {:?}", a);
        if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) = a {
            *base_type = path.clone();
            // println!("BASE_TYPE = {:?}", base_type);

            if let Some(seg) = path.segments.last() {
                match seg.ident.to_string().as_str() {
                    "Option" => *number_of_options += 1,
                    "Vec" => *type_hint = TypeHint::Merge,
                    "Join" => *type_hint = TypeHint::Join,
                    _ => {}
                }
                match &seg.arguments {
                    syn::PathArguments::None => {
                        *base_name = seg.ident.clone();
                    }
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }) => {
                        eval_arguments(
                            args.into_iter(),
                            number_of_options,
                            type_hint,
                            base_name,
                            base_type,
                        )?;
                    }
                    _ => return Err(DeriveError::InvalidType(seg.span())),
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{get_type_info, TypeHint};
    use syn::parse_str;

    #[test]
    fn fields() {
        let ti = get_type_info(&parse_str::<syn::Path>("Other").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 0);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Other);

        let ti = get_type_info(&parse_str::<syn::Path>("Option<Other>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 1);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Other);

        let ti = get_type_info(&parse_str::<syn::Path>("Option<Option<Other>>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 2);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Other);
    }
    #[test]
    fn joins() {
        let ti = get_type_info(&parse_str::<syn::Path>("Join<Other>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 0);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Join);

        let ti = get_type_info(&parse_str::<syn::Path>("Option<Join<Other>>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 1);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Join);

        let ti =
            get_type_info(&parse_str::<syn::Path>("Option<Option<Join<Other>>>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 2);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Join);
    }
    #[test]
    fn merges() {
        let ti = get_type_info(&parse_str::<syn::Path>("Vec<Other>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 0);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Merge);

        let ti = get_type_info(&parse_str::<syn::Path>("Option<Vec<Other>>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 1);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Merge);

        // This is not supported, but can be parsed
        let ti =
            get_type_info(&parse_str::<syn::Path>("Option<Option<Vec<Other>>>").unwrap()).unwrap();
        assert_eq!(ti.number_of_options, 2);
        assert_eq!(ti.base_name.to_string(), "Other");
        assert_eq!(ti.type_hint, TypeHint::Merge);
    }
}
