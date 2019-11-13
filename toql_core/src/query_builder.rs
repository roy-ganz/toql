//! Trait to associate a field type provider with a struct.

/// Used by code produced from Toql derive.
pub trait QueryFields {
    type FieldsType;

    fn fields() -> Self::FieldsType;
    fn fields_from_path(path: String) -> Self::FieldsType;
}

/// Trait to build a Toql query from a Key
pub trait KeyPredicate {
    fn key_predicate(&self) -> crate::query::Query;
}

/*
pub trait QueryFunctions {
    fn key_predicate<K>(key: K::Key) ->Result<crate::query::Query , crate::error::ToqlError>
    where K: crate::key::Key<Key = T>, T: std::hash::Hash + std::cmp::Eq
    ;
} */
