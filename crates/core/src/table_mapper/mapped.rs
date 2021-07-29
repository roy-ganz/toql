use super::TableMapper;

pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn type_name() -> String;
    fn map(mapper: &mut TableMapper) -> crate::result::Result<()>; // Map entity fields
}
