use crate::result::Result;
use crate::{
    sql_mapper::mapped::Mapped, sql_mapper_registry::SqlMapperRegistry, tree::tree_map::TreeMap,
};

pub(crate) fn map<T: Mapped + TreeMap>(registry: &mut SqlMapperRegistry) -> Result<()> {
    if !registry
        .mappers
        .contains_key(<T as Mapped>::type_name().as_str())
    {
        <T as TreeMap>::map(registry)?;
    }
    Ok(())
}
