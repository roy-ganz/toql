use proc_macro2::Span;
#[derive(Debug)]
pub enum DeriveError {
    InvalidEnumValue(syn::LitStr, Vec<String>), // Invalid enum value at span, allowed values
    UnknownAttribute(syn::Ident, Vec<String>),  // Unknown identifier at span, allowed values
    AttributeMissing(syn::Ident, String),       // Expected identifier
    DuplicateAttribute(syn::Ident),             // Duplicated identifier at span
    StringExpected(syn::Lit),                   // Expected string at span
    PathExpected(syn::Lit),                     // Expected string at span
    IntegerExpected(syn::Lit),                  // Expected integer u64 at span
    UnsupportedToken(Span, String),             // Expected integer u64 at span
    InvalidAttribute(Span, String),             // Expected integer u64 at span
    AttributeNameExpected(Span),                // Expected integer u64 at span
    KeyMissing(Span),                           //
    KeyTrailing(Span),                          // Expected integer u64 at span
    OptionalKey(Span),                          // Expected integer u64 at span
    InvalidFieldType(Span),                     // Expected integer u64 at span
    InvalidJoinType(Span),                      // Expected integer u64 at span
    InvalidMergeType(Span),                     // Expected integer u64 at span
}

// Into Syn Error
impl Into<syn::Error> for DeriveError {
    fn into(self) -> syn::Error {
        match self {
            DeriveError::InvalidEnumValue(lit, expected) => {
                let value = lit.value();
                let propose = propose_str(&value, &expected);
                syn::Error::new(
                    lit.span(),
                    match propose {
                        Some(p) => format!("invalid `{}`. Did you mean `{}`?", value, expected[p]),
                        None => format!(
                            "invalid `{}`. Available values are: `{}`",
                            value,
                            expected.join("`,`")
                        ),
                    },
                )
            }
            DeriveError::UnknownAttribute(ident, available) => syn::Error::new(
                ident.span(),
                format!(
                    "unknown attribute `{}`. Available attributes are `{}`",
                    ident,
                    available.join("`,`")
                ),
            ),
            DeriveError::AttributeMissing(ident, expected) => {
                syn::Error::new(ident.span(), format!("expected identifier `{}`", expected))
            }
            DeriveError::DuplicateAttribute(ident) => {
                syn::Error::new(ident.span(), format!("expected duplication of `{}`", ident))
            }
            DeriveError::StringExpected(lit) => {
                syn::Error::new(lit.span(), "expected string".to_string())
            }
            DeriveError::PathExpected(lit) => {
                syn::Error::new(lit.span(), "expected path".to_string())
            }
            DeriveError::IntegerExpected(lit) => {
                syn::Error::new(lit.span(), "expected integer".to_string())
            }
            DeriveError::UnsupportedToken(span, msg) => {
                syn::Error::new(span, format!("unsupported token: {}", msg))
            }
            DeriveError::InvalidAttribute(span, msg) => syn::Error::new(span, msg),
            DeriveError::AttributeNameExpected(span) => {
                syn::Error::new(span, "expected attribute name".to_string())
            }
            DeriveError::KeyMissing(span) => {
                syn::Error::new(span, "key not found in struct. Add `#[toql(key)]` to fields that correspond to primary key.".to_string())
            }
            DeriveError::KeyTrailing(span) => {
                syn::Error::new(span, "key must always be at the beginning of a struct. Move your field.".to_string())
            }
            DeriveError::OptionalKey(span) => {
                syn::Error::new(span, "key must not be optioanl. Remove `Option<T>`.".to_string())
            }
            DeriveError::InvalidFieldType(span) => {
                syn::Error::new(span, "Type is not supported for field.".to_string())
            }
            DeriveError::InvalidJoinType(span) => {
                syn::Error::new(span, "Type is not supported join.".to_string())
            }
            DeriveError::InvalidMergeType(span) => {
                syn::Error::new(span, "Type is not supported for merge.".to_string())
            }
        }
    }
}

pub(crate) fn propose_str(input: &str, options: &[String]) -> Option<usize> {
    for (n, o) in options.iter().enumerate() {
        if levenshtein::levenshtein(input, o) < 4 {
            return Some(n);
        }
    }
    None
}
