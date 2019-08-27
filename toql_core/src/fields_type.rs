

//! Trait to associate a field type provider with a struct.

/// Used by code produced from Toql derive.
pub trait FieldsType {
    type FieldsType;

    
    fn fields() -> Self::FieldsType; 
    fn fields_from_path(path: String) -> Self::FieldsType;
}
