// Trait to associate a field type provider with a struct
// Used by code from derive
pub trait FieldsType {
    type FieldsType;
}
