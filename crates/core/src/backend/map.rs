use crate::result::Result;
use crate::{
    table_mapper::mapped::Mapped, table_mapper_registry::TableMapperRegistry, tree::tree_map::TreeMap,
};

pub(crate) fn map<T: Mapped + TreeMap>(registry: &mut TableMapperRegistry) -> Result<()> {
    if !registry
        .mappers
        .contains_key(<T as Mapped>::type_name().as_str())
    {
        <T as TreeMap>::map(registry)?;
    }
    Ok(())
}
