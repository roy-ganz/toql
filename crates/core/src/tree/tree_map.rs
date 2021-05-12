use crate::{sql_mapper::mapped::Mapped, sql_mapper_registry::SqlMapperRegistry};

// Trait is implemented for structs that can map
pub trait TreeMap: Mapped {
    fn map<'a>(registry: &mut SqlMapperRegistry) -> crate::result::Result<()>;
}
