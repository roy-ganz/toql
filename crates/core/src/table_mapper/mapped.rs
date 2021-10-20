//! `structs that implement this trait, can directly be mapped.
use super::TableMapper;

/// This trait is implemented for every Toql derived `struct`.
/// See documentation on [TableMapper] to see how it is used.
pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn type_name() -> String;
    fn map(mapper: &mut TableMapper) -> crate::result::Result<()>; // Map entity fields
}
