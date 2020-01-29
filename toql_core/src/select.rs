use crate::key::Key;

/// Trait to select entities from database.
/// This is mainly useful for copy or update operations but can also be useful for quick lookups.
pub trait Select<T: Key> {
    type error;

    /// SQL fragment to select columns
    /// (internal use)
    fn columns_sql(alias: &str) -> String;

    /// SQL fragment to select columns
    /// (internal use)
    fn joins_sql() -> String;

    /// SQL statement to select columns
    fn select_sql(join: Option<&str>) -> String;

    /// Select a struct without merge dependencies for a given key.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn select_one(&mut self, key: <T as Key>::Key) -> Result<T, Self::error>;

    fn select_many(&mut self, keys: &[<T as Key>::Key]) -> Result<Vec<T>, Self::error>;
}
