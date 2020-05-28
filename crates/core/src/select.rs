use crate::key::Keyed;




/// Trait to select entities from database.
/// This is mainly useful for copy or update operations but can also be useful for quick lookups.
pub trait Select<T: Keyed> {
    type Error;

    /// SQL fragment to select columns
    /// (internal use)
    fn columns_sql(canonical_alias: &str) -> String;

    /// SQL fragment to select columns
    /// (internal use)
    fn joins_sql(canonical_alias: &str) -> String;

    /// SQL statement to select columns
    fn select_sql(join: Option<&str>) -> String;

    /// Sql alias used for columns
    fn table_alias() -> String;

    /// Select a struct without merge dependencies for a given key.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn select_one(&mut self, key: <T as Keyed>::Key) -> Result<T, Self::Error>;

    fn select_many(&mut self, keys: &[<T as Keyed>::Key]) -> Result<Vec<T>, Self::Error>;
}


