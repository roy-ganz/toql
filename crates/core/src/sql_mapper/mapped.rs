

use crate::sql_mapper::SqlMapper;

/// Structs that implement `Mapped` can be added to the mapper with [map()](struct.SqlMapper.html#method.map).
///
/// The Toql derive implements this trait for derived structs.
pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn type_name() -> String;
    fn map(mapper: &mut SqlMapper, toql_path: &str, sql_alias: &str); // Map entity fields
}
