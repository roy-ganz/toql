//! Trait to associate a field type provider with a struct.

/// Used by code produced from Toql derive.
pub trait QueryFields {
    type FieldsType;

    fn fields() -> Self::FieldsType;
    fn fields_from_path(path: String) -> Self::FieldsType;
}
/*
/// Trait to build a Toql query from a Key or a collection of keys
pub trait KeyPredicate<T: crate::key::Key> {
    fn key_predicate(key: T::Key) -> crate::query::Query {
        Self::key_iter_predicate(std::iter::once(key))

    }
    fn key_iter_predicate<I>(keys: I) -> crate::query::Query
    where
        I: IntoIterator<Item = T::Key>;
}

pub fn key_predicate<K, T>(key: T::Key) -> crate::query::Query
where T: crate::key::Key, K: KeyPredicate<T>
{
    K::key_predicate(key)
}


pub fn key_iter_predicate<K, T, I>(keys: I) -> crate::query::Query
where T: crate::key::Key, K: KeyPredicate<T>, I: IntoIterator<Item = T::Key>,
{
    K::key_iter_predicate(keys)
}  */
