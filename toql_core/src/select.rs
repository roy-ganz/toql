use crate::key::Key;




/// Trait to select entities from database.
/// This is mainly useful for copy or update operations but can also be useful for quick lookups.
pub trait Select<T: Key> {
    type Error;

    /// SQL fragment to select columns
    /// (internal use)
    fn columns_sql(alias: &str) -> String;

    /// SQL fragment to select columns
    /// (internal use)
    fn joins_sql() -> String;

    /// SQL statement to select columns
    fn select_sql(join: Option<&str>) -> String;

    /// Sql alias used for columns
    fn table_alias() -> String;

    /// Select a struct without merge dependencies for a given key.
    ///
    /// Returns a struct or a [ToqlError](../toql_core/error/enum.ToqlError.html) if no struct was found _NotFound_ or more than one _NotUnique_.
    fn select_one(&mut self, key: <T as Key>::Key) -> Result<T, Self::Error>;

    fn select_many(&mut self, keys: &[<T as Key>::Key]) -> Result<Vec<T>, Self::Error>;
}

/// Trait to turn a partial or full key into a sql predicate
pub trait SqlPredicate {
    type Entity; // Output type

    fn sql_predicate(&self, alias:&str) -> (String, Vec<String>);

}


impl<T, U> SqlPredicate for &[U] 
where U: SqlPredicate<Entity =T>
{
    type Entity= T;

     fn sql_predicate(&self, alias:&str) -> (String, Vec<String>){

         let mut predicate = String::new();
         let mut params = Vec::new();

         for i in *self {
             let (pr, pa) = i.sql_predicate(alias);
             predicate.push_str(&pr);
             params.extend_from_slice(&pa);
         }
         (predicate, params)
     }
}

