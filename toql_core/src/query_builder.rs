//! Trait to associate a field type provider with a struct.

/// Used by code produced from Toql derive.
pub trait QueryFields {
    type FieldsType;

    fn fields() -> Self::FieldsType;
    fn fields_from_path(path: String) -> Self::FieldsType;
}


pub trait Query {
    fn key_predicate<T>(key: &<T as crate::key::Key<T>>::Key) ->Result<crate::query::Query , crate::error::ToqlError>
    where T : crate::key::Key<T>;

}