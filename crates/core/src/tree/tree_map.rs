//! Trait to map entities. 
use crate::{table_mapper::mapped::Mapped, table_mapper_registry::TableMapperRegistry};

/// The trait maps the implementing struct and its dependencies.
/// It will insert the struct in a [TableMapper](crate::table_mapper::TableMapper)
/// and register the mapper in the [TableMapperRegistry](crate::table_mapper_registry::TableMapperRegistry).
///
/// Trait is implemented by the Toql derive for structs that can map.
pub trait TreeMap: Mapped {
    fn map(registry: &mut TableMapperRegistry) -> crate::result::Result<()>;
}
