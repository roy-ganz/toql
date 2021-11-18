use crate::parsed::{field::field_kind::FieldKind, parsed_struct::ParsedStruct};
use proc_macro2::TokenStream;

pub(crate) fn to_tokens(parsed_struct: &ParsedStruct, tokens: &mut TokenStream) {
    let mut dispatch_map_code = Vec::new();

    let struct_name = &parsed_struct.struct_name.to_string();
    let struct_name_ident = &parsed_struct.struct_name;

    for field in &parsed_struct.fields {
        let field_base_type = &field.field_base_type;

        match &field.kind {
            FieldKind::Skipped => {}
            FieldKind::Regular(_) => {}
            FieldKind::Join(_) => {
                dispatch_map_code.push(quote!(
                            <#field_base_type as toql::tree::tree_map::TreeMap>::map(registry)?;
                ));
            }
            FieldKind::Merge(_) => {
                dispatch_map_code.push(quote!(
                            <#field_base_type as toql::tree::tree_map::TreeMap>::map(registry)?;
                ));
            }
        }
    }

    let insert_mapper_code = if let Some(handler) = &parsed_struct.field_handler {
        quote!(registry.insert_new_mapper_with_handler::<#struct_name_ident, _>(#handler ())?;)
    } else {
        quote!(registry.insert_new_mapper::<#struct_name_ident>()?;)
    };

    // Generate token stream
    let mods = quote! {
        impl toql::tree::tree_map::TreeMap for #struct_name_ident {
            fn map(registry: &mut toql::table_mapper_registry::TableMapperRegistry)-> toql::result::Result<()>{
                if registry.get(#struct_name).is_none() {
                    #insert_mapper_code
                }
                #(#dispatch_map_code)*
                Ok(())
            }
        }
        impl toql::tree::tree_map::TreeMap for &#struct_name_ident {
            fn map(registry: &mut toql::table_mapper_registry::TableMapperRegistry)-> toql::result::Result<()>{
                <#struct_name_ident as  toql::tree::tree_map::TreeMap>::map(registry)
            }
        }
        impl toql::tree::tree_map::TreeMap for &mut #struct_name_ident {
            fn map(registry: &mut toql::table_mapper_registry::TableMapperRegistry)-> toql::result::Result<()>{
                <#struct_name_ident as  toql::tree::tree_map::TreeMap>::map(registry)
            }
        }
    };

    log::debug!("Source code for `{}`:\n{}", struct_name, mods.to_string());
    tokens.extend(mods);
}
