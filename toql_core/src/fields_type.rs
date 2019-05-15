

//! Trait to associate a field type provider with a struct.

/// Used by code produced from Toql derive.
pub trait FieldsType {
    type FieldsType;
}
