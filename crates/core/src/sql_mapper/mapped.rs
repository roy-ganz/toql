use super::SqlMapper;

pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn type_name() -> String;
    fn map(mapper: &mut SqlMapper, toql_path: &str) -> crate::error::Result<()>; // Map entity fields
}
