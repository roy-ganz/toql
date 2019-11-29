use mysql::prelude::GenericConnection;
use toql_core::error::ToqlError;
use toql_core::key::Key;

/// Trait to select entities from MySQL database.
pub trait Select<T: Key> {
    /// SQL fragment to select columns
    /// (internal use)
    fn columns_sql(alias: &str) -> String;

    /// SQL fragment to select columns
    /// (internal use)
    fn joins_sql() -> String;

    /// SQL statement to select columns
    fn select_sql(join: Option<&str>) -> String;

    /// Select a struct with all dependencies for a given key.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn select_one<C: GenericConnection>(
        key: <T as Key>::Key,
        conn: &mut C,
    ) -> Result<T, ToqlError>;

    /// Select a vector of structs with all dependencies for a given key.
    ///
    /// Returns a tuple with the structs.
    fn select_many<C: GenericConnection>(
        key: &<T as Key>::Key,
        conn: &mut C,
        first: u64,
        max: u16,
    ) -> Result<Vec<T>, ToqlError>;

    /// Select a vector of structs with all dependencies for a given JOIN clause.
    /// This function is used internally to fetch merged fields.
    /// Returns a tuple with the structs.
    fn select_dependencies<C: GenericConnection>(
        join: &str,
        params: &Vec<String>,
        conn: &mut C,
    ) -> Result<Vec<T>, ToqlError>;
}
