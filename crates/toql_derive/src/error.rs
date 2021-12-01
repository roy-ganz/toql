use std::fmt;

use proc_macro2::Span;
#[derive(Debug)]
pub enum DeriveError {
    // Expected an attribute
    AttributeExpected(Span),
    // Duplicated identifier at span
    AttributeDuplicate(Span),
    // Unknown attribute, contains allowed attribute names
    AttributeUnknown(Span, String),
    // Attribute is not used correctly
    AttributeInvalid(Span),
    // Required attribute is missing
    AttributeRequired(Span, String),
    // Attribute value is unknown, contains valid attribute values
    AttributeValueUnknown(Span, String),
    // Attribute value is invalid
    AttributeValueInvalid(Span),
    // Key is missing
    KeyMissing(Span),
    // Key is not at the beginning of a struct
    KeyTrailing(Span),
    // Key must not be optional
    OptionalKey(Span),
    // Type is not supported for field, join or merge
    InvalidType(Span),
    // Any other error happened, contains error message
    Custom(Span, String),
}

pub(crate) fn attribute_err(span: Span, keyword: &str, keywords: &[&str]) -> DeriveError {
    if keywords.contains(&keyword) {
        DeriveError::AttributeInvalid(span)
    } else {
        DeriveError::AttributeUnknown(span, keywords.join(", "))
    }
}

impl Into<syn::Error> for DeriveError {
    fn into(self) -> syn::Error {
        syn::Error::new(self.span(), self.to_string())
    }
}

impl DeriveError {
    pub fn span(&self) -> Span {
        match self {
            DeriveError::AttributeExpected(span)
            | DeriveError::AttributeDuplicate(span)
            | DeriveError::AttributeUnknown(span, _)
            | DeriveError::AttributeInvalid(span)
            | DeriveError::AttributeRequired(span, _)
            | DeriveError::AttributeValueUnknown(span, _)
            | DeriveError::AttributeValueInvalid(span)
            | DeriveError::KeyMissing(span)
            | DeriveError::KeyTrailing(span)
            | DeriveError::OptionalKey(span)
            | DeriveError::InvalidType(span)
            | DeriveError::Custom(span, _) => span.to_owned(),
        }
    }
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeriveError::AttributeExpected(_) => writeln!(f, "expected attribute"),
            DeriveError::AttributeDuplicate(_) => write!(f, "attribute must be used only once"),
            DeriveError::AttributeUnknown(_, available) => write!(f, "unknown attribute. Available attributes are `{}`", available),
            DeriveError::AttributeInvalid(_) => write!(f, "attribute is known, but not used correctly"),
            DeriveError::AttributeRequired(_, name) => write!(f, "attribute `{}` is required", name),
            DeriveError::AttributeValueUnknown(_, expected) => write!(f, "invalid value. Available values are: `{}`",expected),
            DeriveError::AttributeValueInvalid(_) => write!(f, "invalid value"),
            DeriveError::KeyMissing(_) => write!(f, "key not found in struct. Add `#[toql(key)]` to fields that correspond to primary key."),
            DeriveError::KeyTrailing(_) => write!(f, "key must always be at the beginning of a struct. Move your field."),
            DeriveError::OptionalKey(_) => write!(f, "key must not be optional."),
            DeriveError::InvalidType(_) => write!(f, "Type is not supported."),
            DeriveError::Custom(_, message) => write!(f, "{}", message),
        }
    }
}
