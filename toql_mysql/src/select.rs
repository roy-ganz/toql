use mysql::Conn;
use toql_core::error::ToqlError;
use toql_core::key::Key;

/// Trait to select entities from MySQL database.
pub trait Select<T :Key<T>> {
   
    /// SQL fragment to select columns
    /// (internal use)
    fn columns_sql() -> String;
    
    /// SQL fragment to select columns
    /// (internal use)
    fn joins_sql() -> String;

    /// SQL statement to select columns
    fn select_sql(join:Option<&str>) -> String;

    /// Select a struct with all dependencies for a given key.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn select_one(key: &<T as Key<T>>::Key, conn: &mut Conn) -> Result<T, ToqlError>;

    /// Select a vector of structs with all dependencies for a given key.
    ///
    /// Returns a tuple with the structs.
    fn select_many(
        key: &<T as Key<T>>::Key,
        conn: &mut Conn,
        first: u64,
        max: u16
    ) -> Result<Vec<T> , ToqlError>;

    /// Select a vector of structs with all dependencies for a given JOIN clause.
    /// This function is used internally to fetch merged fields.
    /// Returns a tuple with the structs.
    fn select_dependencies(join: &str, params: &Vec<String>,conn: &mut Conn) -> Result<Vec<T> , ToqlError>;
  
}
