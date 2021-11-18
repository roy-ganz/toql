pub(crate) mod field_kind;
pub(crate) mod integrity;
pub(crate) mod join_field;
pub(crate) mod merge_field;
pub(crate) mod param_arg;
pub(crate) mod regular_field;
pub(crate) mod type_info;

use crate::{
    attr::{field_attr::FieldAttr, struct_attr::StructAttr},
    result::Result,
};
use field_kind::FieldKind;
use syn::Ident;

#[derive(Debug, Default)]
pub(crate) struct FieldRoles {
    pub(crate) load: Option<String>,
    pub(crate) update: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) field_name: Ident,
    pub(crate) field_type: syn::Path,
    pub(crate) field_base_type: syn::Path,
    pub(crate) field_base_name: syn::Ident,
    pub(crate) toql_query_name: String,
    pub(crate) roles: FieldRoles,
    pub(crate) kind: FieldKind,
    pub(crate) skip_mut: bool,
    pub(crate) skip: bool,
}

impl Field {
    pub(crate) fn try_from(struct_attr: &StructAttr, field_attr: FieldAttr) -> Result<Field> {
        use heck::MixedCase;

        let toql_query_name = field_attr.name.to_string().to_mixed_case();

        // Determine type of field
        // 1. type contains `Join` it must be join
        // 2. type contains `Vec` it must be merge
        // 3. join attribute is set, it is join
        // 4. merge attribute is set, it is merge
        // 5. field must be regular field
        let type_info = type_info::get_type_info(&field_attr.type_path)?;

        let kind = field_kind::build(&struct_attr, &field_attr, &type_info)?;
        let field = Field {
            field_name: field_attr.name,
            field_type: field_attr.type_path,
            field_base_name: type_info.base_name,
            field_base_type: type_info.base_type,
            toql_query_name,
            roles: field_attr.roles,
            kind,
            skip_mut: field_attr.skip_mut.unwrap_or_default(),
            skip: field_attr.skip.unwrap_or_default(),
        };

        //   println!("FIELD = {:?}", &field);

        Ok(field)
    }
}
