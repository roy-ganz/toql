use crate::{table_mapper::mapped::Mapped, table_mapper_registry::TableMapperRegistry};

// Trait is implemented for structs that can map
pub trait TreeMap: Mapped {
    fn map(registry: &mut TableMapperRegistry) -> crate::result::Result<()>;
}
