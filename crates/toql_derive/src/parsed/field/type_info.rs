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
        //println!("SEG = {:?}", &type_path);
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
            _ => {
                return Err(DeriveError::UnsupportedToken(
                    seg.span(),
                    "unexpected parenthesis".to_string(),
                ))
            }
        }
    }

    Err(DeriveError::UnsupportedToken(
        type_path.span(),
        "unexpected parenthesis".to_string(),
    ))
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
                    _ => {
                        return Err(DeriveError::UnsupportedToken(
                            seg.span(),
                            "unexpected parenthesis".to_string(),
                        ))
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::get_type_info;
    use syn::parse_str;

    #[test]
    fn fields() {
        let ti = get_type_info(&parse_str::<syn::Path>("Other").unwrap()).unwrap();

        assert_eq!(ti.number_of_options, 0);
        assert_eq!(ti.base_name.to_string(), "Other");
    }
}
