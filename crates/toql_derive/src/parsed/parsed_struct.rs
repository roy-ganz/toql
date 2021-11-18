use super::{
    field::Field, predicate_arg::PredicateArg, rename_case::RenameCase, selection_arg::SelectionArg,
};
use std::collections::HashMap;
use syn::{Ident, Path};

#[derive(Debug, Default, Clone)]
pub(crate) struct StructRoles {
    pub(crate) load: Option<String>,
    pub(crate) update: Option<String>,
    pub(crate) insert: Option<String>,
    pub(crate) delete: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ParsedStruct {
    /// Visibility of derived struct
    pub(crate) vis: syn::Visibility,
    /// Name of derived struct
    pub(crate) struct_name: Ident,
    /// Table rename scheme (includes joined, merged tables)
    pub(crate) tables: RenameCase,
    /// Table name of the struct
    pub(crate) table: String,
    /// Columns rename scheme (include auto columns of joins and merges)
    pub(crate) columns: RenameCase,
    /// Struct is readonly and can't be inserted or updated
    pub(crate) skip_mut: bool,
    // Struct key is generated in database and should be refreshed in rust
    pub(crate) auto_key: bool,
    // Predicates for that struct
    pub(crate) predicates: HashMap<String, PredicateArg>,
    // Selections for that struct
    pub(crate) selections: HashMap<String, SelectionArg>,

    // Role restrictions for that struct
    pub(crate) roles: StructRoles,
    // Toql relevant fields on that struct
    pub(crate) fields: Vec<Field>,
    //  Default field handler
    pub(crate) field_handler: Option<Path>,
}
