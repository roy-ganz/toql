use std::ops::Deref;

use syn::NestedMeta;

use darling::{Error, FromMeta, Result};
use std::collections::HashSet;

/// A list of `syn::String` instances. This type is used to extract a list of paths from an
/// attribute.
///
/// # Usage
/// An `StringSet` field on a struct implementing `FromMeta` will turn `#[builder(derive(serde::Debug, Clone))]` into:
///
/// ```rust,ignore
/// StructOptions {
///     derive: StringSet(vec![syn::String::new("serde::Debug"), syn::String::new("Clone")])
/// }
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StringSet(pub HashSet<String>);

impl Deref for StringSet {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HashSet<String>> for StringSet {
    fn from(v: HashSet<String>) -> Self {
        StringSet(v)
    }
}

impl FromMeta for StringSet {
    fn from_list(v: &[NestedMeta]) -> Result<Self> {
        let mut paths = HashSet::with_capacity(v.len());
        for nmi in v {
            if let NestedMeta::Literal(syn::Lit::Str(ref string)) = *nmi {
                paths.insert(string.value());
            } else {
                return Err(Error::unexpected_type("non-string").with_span(nmi));
            }
        }

        Ok(StringSet(paths))
    }
}
